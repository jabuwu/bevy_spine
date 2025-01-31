//! A Bevy plugin for Spine 4.2
//!
//! Add [`SpinePlugin`] to your Bevy app and spawn a [`SpineBundle`] to get started!

use std::{
    collections::{HashMap, VecDeque},
    mem::take,
    sync::{Arc, Mutex},
};

use bevy::{
    asset::load_internal_binary_asset,
    image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
    prelude::*,
    render::{
        mesh::{Indices, MeshVertexAttribute},
        render_asset::RenderAssetUsages,
        render_resource::{PrimitiveTopology, VertexFormat},
    },
    sprite::Material2dPlugin,
};
use materials::{
    SpineAdditiveMaterial, SpineAdditivePmaMaterial, SpineMaterialInfo, SpineMultiplyMaterial,
    SpineMultiplyPmaMaterial, SpineNormalMaterial, SpineNormalPmaMaterial, SpineScreenMaterial,
    SpineScreenPmaMaterial,
};
use rusty_spine::{
    atlas::{AtlasFilter, AtlasWrap},
    controller::{SkeletonCombinedRenderable, SkeletonRenderable},
    AnimationEvent, Physics, Skeleton,
};
use textures::SpineTextureConfig;

use crate::{
    assets::{AtlasLoader, SkeletonJsonLoader},
    materials::{SpineMaterialPlugin, DARK_COLOR_ATTRIBUTE, SHADER_HANDLE},
    rusty_spine::{
        controller::SkeletonControllerSettings, draw::CullDirection, AnimationStateData, BoneHandle,
    },
    textures::{SpineTexture, SpineTextureCreateEvent, SpineTextureDisposeEvent, SpineTextures},
};

pub use crate::{assets::*, crossfades::Crossfades, entity_sync::*, handle::*, rusty_spine::Color};

/// See [`rusty_spine`] docs for more info.
pub use crate::rusty_spine::controller::SkeletonController;

pub use rusty_spine;

/// System sets for Spine systems.
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, SystemSet)]
pub enum SpineSystem {
    /// Loads [`SkeletonData`] assets which must exist before a [`SpineBundle`] can fully load.
    Load,
    /// Spawns helper entities associated with a [`SpineBundle`] for drawing meshes and
    /// (optionally) adding bone entities (see [`SpineLoader`]).
    Spawn,
    /// An [`apply_deferred`] to load the spine helper entities this frame.
    SpawnFlush,
    /// Sends [`SpineReadyEvent`] after [`SpineSystem::SpawnFlush`], indicating [`Spine`] components
    /// on newly spawned [`SpineBundle`]s can now be interacted with.
    Ready,
    /// Advances all animations and processes Spine events (see [`SpineEvent`]).
    UpdateAnimation,
    /// Updates all Spine meshes.
    UpdateMeshes,
    /// Updates all Spine materials.
    UpdateMaterials,
    /// Adjusts Spine textures to render properly.
    AdjustSpineTextures,
}

/// Helper sets for interacting with Spine systems.
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, SystemSet)]
pub enum SpineSet {
    /// A helper Set occuring after [`SpineSystem::Ready`] but before Spine update systems, so that
    /// systems can configure a newly spawned skeleton before they are updated for the first time.
    OnReady,
    /// A helper Set occuring after [`SpineSystem::UpdateAnimation`] but before
    /// [`SpineSystem::UpdateMeshes`], so that systems can handle events immediately after the
    /// skeleton updates but before it renders.
    OnEvent,
    /// A helper set occuring simultaneously with [`SpineSystem::UpdateMeshes`], useful for custom
    /// mesh creation when using [`SpineDrawer::None`].
    OnUpdateMesh,
}

/// Add Spine support to Bevy!
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_spine::SpinePlugin;
/// # fn doc() {
/// App::new()
///     .add_plugins(DefaultPlugins)
///     .add_plugins(SpinePlugin)
///     // ...
///     .run();
/// # }
/// ```
pub struct SpinePlugin;

impl Plugin for SpinePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            Material2dPlugin::<SpineNormalMaterial>::default(),
            Material2dPlugin::<SpineAdditiveMaterial>::default(),
            Material2dPlugin::<SpineMultiplyMaterial>::default(),
            Material2dPlugin::<SpineScreenMaterial>::default(),
            Material2dPlugin::<SpineNormalPmaMaterial>::default(),
            Material2dPlugin::<SpineAdditivePmaMaterial>::default(),
            Material2dPlugin::<SpineMultiplyPmaMaterial>::default(),
            Material2dPlugin::<SpineScreenPmaMaterial>::default(),
        ))
        .add_plugins((
            SpineMaterialPlugin::<SpineNormalMaterial>::default(),
            SpineMaterialPlugin::<SpineAdditiveMaterial>::default(),
            SpineMaterialPlugin::<SpineMultiplyMaterial>::default(),
            SpineMaterialPlugin::<SpineScreenMaterial>::default(),
            SpineMaterialPlugin::<SpineNormalPmaMaterial>::default(),
            SpineMaterialPlugin::<SpineAdditivePmaMaterial>::default(),
            SpineMaterialPlugin::<SpineMultiplyPmaMaterial>::default(),
            SpineMaterialPlugin::<SpineScreenPmaMaterial>::default(),
        ))
        .add_plugins(SpineSyncPlugin::first())
        .init_resource::<SpineEventQueue>()
        .insert_resource(SpineTextures::init())
        .insert_resource(SpineReadyEvents::default())
        .add_event::<SpineTextureCreateEvent>()
        .add_event::<SpineTextureDisposeEvent>()
        .init_asset::<Atlas>()
        .init_asset::<SkeletonJson>()
        .init_asset::<SkeletonBinary>()
        .init_asset::<SkeletonData>()
        .init_asset_loader::<AtlasLoader>()
        .init_asset_loader::<SkeletonJsonLoader>()
        .init_asset_loader::<SkeletonBinaryLoader>()
        .add_event::<SpineReadyEvent>()
        .add_event::<SpineEvent>()
        .add_systems(
            Update,
            (
                spine_load.in_set(SpineSystem::Load),
                spine_spawn
                    .in_set(SpineSystem::Spawn)
                    .after(SpineSystem::Load),
                spine_ready
                    .in_set(SpineSystem::Ready)
                    .after(SpineSystem::Spawn)
                    .before(SpineSet::OnReady),
                spine_update_animation
                    .in_set(SpineSystem::UpdateAnimation)
                    .after(SpineSet::OnReady)
                    .before(SpineSet::OnEvent),
                spine_update_meshes
                    .in_set(SpineSystem::UpdateMeshes)
                    .in_set(SpineSet::OnUpdateMesh)
                    .after(SpineSystem::UpdateAnimation)
                    .after(SpineSet::OnEvent),
                apply_deferred
                    .in_set(SpineSystem::SpawnFlush)
                    .after(SpineSystem::Spawn)
                    .before(SpineSystem::Ready),
            ),
        )
        .add_systems(
            PostUpdate,
            adjust_spine_textures.in_set(SpineSystem::AdjustSpineTextures),
        );

        load_internal_binary_asset!(
            app,
            SHADER_HANDLE,
            "spine.wgsl",
            |bytes: &[u8], path: String| Shader::from_wgsl(
                std::str::from_utf8(bytes).unwrap().to_owned(),
                path
            )
        );
    }
}

#[derive(Resource, Default)]
struct SpineEventQueue(Arc<Mutex<VecDeque<SpineEvent>>>);

/// A live Spine [`SkeletonController`] [`Component`], ready to be manipulated.
///
/// This component does not exist on [`SpineBundle`] initially, since Spine assets may not yet be
/// loaded when an entity is spawned. Querying for this component type guarantees that all entities
/// containing it have a Spine rig that is ready to use.
#[derive(Component, Debug)]
pub struct Spine(pub SkeletonController);

/// When loaded, a [`Spine`] entity has children entities attached to it, each containing this
/// component.
///
/// To disable creation of these child entities, see [`SpineLoader::without_children`].
///
/// The bones are not automatically synchronized, but can be synchronized easily by adding a
/// [`SpineSync`] component.
#[derive(Component, Debug)]
pub struct SpineBone {
    pub spine_entity: Entity,
    pub handle: BoneHandle,
    pub name: String,
    pub parent: Option<SpineBoneParent>,
}

#[derive(Debug)]
pub struct SpineBoneParent {
    pub entity: Entity,
    pub handle: BoneHandle,
}

#[derive(Component, Clone)]
pub struct SpineMeshes;

/// Marker component for child entities containing [`Mesh`] components for Spine rendering.
///
/// By default, the meshes may contain several meshes all combined into one to reduce draw calls
/// and improve performance. To interact with individual Spine meshes, see
/// [`SpineSettings::drawer`].
#[derive(Component, Debug, Clone)]
pub struct SpineMesh {
    pub spine_entity: Entity,
    pub handle: Handle<Mesh>,
    pub state: SpineMeshState,
}

/// The state of this [`SpineMesh`].
#[derive(Default, Component, Debug, Clone)]
pub enum SpineMeshState {
    /// This Spine mesh contains no mesh data and should not render.
    #[default]
    Empty,
    /// This Spine mesh contains mesh data and should render.
    Renderable { info: SpineMaterialInfo },
}

impl core::ops::Deref for Spine {
    type Target = SkeletonController;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for Spine {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// The async loader for Spine assets. Waits for Spine assets to be ready in the [`AssetServer`],
/// then initializes child entities, and finally attaches the live [`Spine`] component.
///
/// When spawning a [`SpineLoader`] (typically through [`SpineBundle`]), it will create child
/// entities representing the bones of a skeleton (see [`SpineBone`]). These bones are not
/// synchronized (see [`SpineSync`]), and can be disabled entirely using
/// [`SpineLoader::without_children`].
#[derive(Component, Debug)]
pub enum SpineLoader {
    /// The spine rig is still loading.
    Loading {
        /// If true, will spawn child entities for each bone in the skeleton (see [`SpineBone`]).
        with_children: bool,
    },
    /// The spine rig is ready.
    Ready,
    /// The spine rig failed to load.
    Failed,
}

impl Default for SpineLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl SpineLoader {
    pub fn new() -> Self {
        Self::with_children()
    }

    pub fn with_children() -> Self {
        Self::Loading {
            with_children: true,
        }
    }

    /// Load a [`Spine`] entity without child entities containing [`SpineBone`] components.
    ///
    /// Renderable mesh child entities are still created.
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_spine::{SpineLoader, SpineBundle};
    /// # fn doc(mut commands: Commands) {
    /// commands.spawn(SpineBundle {
    ///     // ..
    ///     loader: SpineLoader::without_children(),
    ///     ..Default::default()
    /// });
    /// # }
    /// ```
    pub fn without_children() -> Self {
        Self::Loading {
            with_children: false,
        }
    }
}

/// Settings for how this Spine updates and renders.
///
/// Typically set in [`SpineBundle`] when spawning an entity.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpineSettings {
    /// Indicates if default Spine materials should be used (default: `true`).
    ///
    /// If `false`, a custom [`SpineMaterial`](`materials::SpineMaterial`) should be configured for
    /// this Spine.
    pub default_materials: bool,
    /// Indicates how the meshes should be drawn.
    pub mesh_type: SpineMeshType,
    /// The drawer this Spine should use to create its meshes.
    pub drawer: SpineDrawer,
}

/// Mesh types to use in [`SpineSettings`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpineMeshType {
    /// Render meshes in 2D.
    Mesh2D,
    /// Render meshes in 3D. Requires a custom [`SpineMaterial`](`materials::SpineMaterial`) since
    /// the default materials do not support 3D meshes.
    Mesh3D,
}

/// Drawer methods to use in [`SpineSettings`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpineDrawer {
    /// Draw each slot as a separate mesh, each represented by one [`SpineMesh`].
    ///
    /// Useful if individual meshes need separate materials, z-depth, or other rendering
    /// differences. Less performant, but more versatile than [`SpineDrawer::Combined`].
    Separated,
    /// Combine multiple slots into a single mesh.
    ///
    /// The default, and most performanent drawer method. Suitable for most use cases.
    Combined,
    /// Do not update meshes at all.
    None,
}

impl Default for SpineSettings {
    fn default() -> Self {
        Self {
            default_materials: true,
            mesh_type: SpineMeshType::Mesh2D,
            drawer: SpineDrawer::Combined,
        }
    }
}

/// Bundle for Spine skeletons with all the necessary components.
///
/// See [`SkeletonData::new_from_json`] or [`SkeletonData::new_from_binary`] for example usages.
///
/// Note that this bundle does not contain the [`Spine`] component itself, which is the primary way
/// to query and interact with Spine skeletons. Instead, a [`SpineLoader`] is added which ensures
/// that all the necessary assets ([`Atlas`] and [`SkeletonJson`]/[`SkeletonBinary`]) are loaded
/// before instantiating the Spine skeleton. This ensures that querying for [`Spine`] components
/// will always yield fully instantiated skeletons.
///
/// It is possible to spawn a Spine skeleton and initialize it in the same frame. To do so, ensure
/// that the spawning system occurs before [`SpineSystem::Spawn`] and the initializing system is in
/// the [`SpineSet::OnReady`] set (assuming the [`SkeletonData`] has already been loaded). Listen
/// for [`SpineReadyEvent`] to get newly loaded skeletons.
///
/// ```
/// use bevy::prelude::*;
/// use bevy_spine::prelude::*;
///
/// # let mut app = App::new();
/// {
///     // in main() or a plugin
///     app.add_systems(
///         Update,
///         (
///             spawn_spine.before(SpineSystem::Spawn),
///             init_spine.in_set(SpineSet::OnReady),
///         ),
///     );
/// }
///
/// #[derive(Resource)]
/// struct MyGameAssets {
///     // loaded ahead of time
///     skeleton: Handle<SkeletonData>
/// }
///
/// #[derive(Component)]
/// struct MySpine;
///
/// fn spawn_spine(
///     mut commands: Commands,
///     my_game_assets: Res<MyGameAssets>
/// ) {
///     commands.spawn((
///         SpineBundle {
///             skeleton: my_game_assets.skeleton.clone(),
///             ..Default::default()
///         },
///         MySpine
///     ));
/// }
///
/// fn init_spine(
///     mut spine_ready_events: EventReader<SpineReadyEvent>,
///     mut spine_query: Query<&mut Spine, With<MySpine>>
/// ) {
///     for spine_ready_event in spine_ready_events.read() {
///         if let Ok(mut spine) = spine_query.get_mut(spine_ready_event.entity) {
///             // the skeleton will start playing the animation the same frame it spawns on
///             spine.animation_state.set_animation_by_name(0, "animation", true);
///         }
///     }
/// }
/// ```
#[derive(Default, Bundle)]
pub struct SpineBundle {
    pub loader: SpineLoader,
    pub settings: SpineSettings,
    pub skeleton: SkeletonDataHandle,
    pub crossfades: Crossfades,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

/// An [`Event`] which is sent once a [`SpineLoader`] has fully loaded a skeleton and attached the
/// [`Spine`] component.
///
/// For convenience, systems receiving this event can be added to the [`SpineSet::OnReady`] set to
/// receive this after events are sent, but before the first [`SkeletonController`] update.
#[derive(Debug, Clone, Event)]
pub struct SpineReadyEvent {
    /// The entity containing the [`Spine`] component.
    pub entity: Entity,
    /// A list of all bones (if spawned, see [`SpineBone`]).
    pub bones: HashMap<String, Entity>,
}

/// A Spine event fired from a playing animation.
///
/// Sent in [`SpineSystem::UpdateAnimation`].
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_spine::prelude::*;
/// // bevy system
/// fn on_spine_event(
///     mut spine_events: EventReader<SpineEvent>,
///     mut commands: Commands,
///     asset_server: Res<AssetServer>,
/// ) {
///     for event in spine_events.read() {
///         if let SpineEvent::Event { name, entity, .. } = event {
///             println!("spine event fired: {}", name);
///             println!("from entity: {:?}", entity);
///         }
///     }
/// }
/// ```
#[derive(Debug, Clone, Event)]
pub enum SpineEvent {
    Start {
        entity: Entity,
        animation: String,
    },
    Interrupt {
        entity: Entity,
        animation: String,
    },
    End {
        entity: Entity,
        animation: String,
    },
    Complete {
        entity: Entity,
        animation: String,
    },
    Dispose {
        entity: Entity,
    },
    Event {
        entity: Entity,
        name: String,
        int: i32,
        float: f32,
        string: String,
        audio_path: String,
        volume: f32,
        balance: f32,
    },
}

/// Queued ready events, to be sent after [`SpineSystem::SpawnFlush`].
#[derive(Default, Resource)]
struct SpineReadyEvents(Vec<SpineReadyEvent>);

#[allow(clippy::too_many_arguments)]
fn spine_load(
    mut skeleton_data_assets: ResMut<Assets<SkeletonData>>,
    mut texture_create_events: EventWriter<SpineTextureCreateEvent>,
    mut texture_dispose_events: EventWriter<SpineTextureDisposeEvent>,
    atlases: Res<Assets<Atlas>>,
    jsons: Res<Assets<SkeletonJson>>,
    binaries: Res<Assets<SkeletonBinary>>,
    spine_textures: Res<SpineTextures>,
    asset_server: Res<AssetServer>,
) {
    // check if any assets are loading, else, early out to avoid triggering change detection
    let mut loading = false;
    for (_, skeleton_data_asset) in skeleton_data_assets.iter() {
        if matches!(skeleton_data_asset.status, SkeletonDataStatus::Loading) {
            loading = true;
            break;
        }
    }
    if loading {
        for (_, skeleton_data_asset) in skeleton_data_assets.iter_mut() {
            let SkeletonData {
                atlas_handle,
                kind,
                status,
                premultiplied_alpha,
            } = skeleton_data_asset;
            if matches!(status, SkeletonDataStatus::Loading) {
                let atlas = if let Some(atlas) = atlases.get(atlas_handle) {
                    atlas
                } else {
                    continue;
                };
                if let Some(page) = atlas.atlas.pages().next() {
                    *premultiplied_alpha = page.pma();
                }
                match kind {
                    SkeletonDataKind::JsonFile(json_handle) => {
                        let json = if let Some(json) = jsons.get(json_handle) {
                            json
                        } else {
                            continue;
                        };
                        let skeleton_json = rusty_spine::SkeletonJson::new(atlas.atlas.clone());
                        match skeleton_json.read_skeleton_data(&json.json) {
                            Ok(skeleton_data) => {
                                *status = SkeletonDataStatus::Loaded(Arc::new(skeleton_data));
                            }
                            Err(_err) => {
                                *status = SkeletonDataStatus::Failed;
                                continue;
                            }
                        }
                    }
                    SkeletonDataKind::BinaryFile(binary_handle) => {
                        let binary = if let Some(binary) = binaries.get(binary_handle) {
                            binary
                        } else {
                            continue;
                        };
                        let skeleton_binary = rusty_spine::SkeletonBinary::new(atlas.atlas.clone());
                        match skeleton_binary.read_skeleton_data(&binary.binary) {
                            Ok(skeleton_data) => {
                                *status = SkeletonDataStatus::Loaded(Arc::new(skeleton_data));
                            }
                            Err(_err) => {
                                // TODO: print error?
                                *status = SkeletonDataStatus::Failed;
                                continue;
                            }
                        }
                    }
                }
            }
        }
    }

    spine_textures.update(
        asset_server.as_ref(),
        atlases.as_ref(),
        &mut texture_create_events,
        &mut texture_dispose_events,
    );
}

#[allow(clippy::too_many_arguments)]
fn spine_spawn(
    mut skeleton_query: Query<(
        &mut SpineLoader,
        Entity,
        &SkeletonDataHandle,
        Option<&Crossfades>,
    )>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut ready_events: ResMut<SpineReadyEvents>,
    mut skeleton_data_assets: ResMut<Assets<SkeletonData>>,
    spine_event_queue: Res<SpineEventQueue>,
) {
    for (mut spine_loader, spine_entity, data_handle, crossfades) in skeleton_query.iter_mut() {
        if let SpineLoader::Loading { with_children } = spine_loader.as_ref() {
            let skeleton_data_asset =
                if let Some(skeleton_data_asset) = skeleton_data_assets.get_mut(&data_handle.0) {
                    skeleton_data_asset
                } else {
                    continue;
                };
            match &skeleton_data_asset.status {
                SkeletonDataStatus::Loaded(skeleton_data) => {
                    let mut animation_state_data = AnimationStateData::new(skeleton_data.clone());
                    if let Some(crossfades) = crossfades {
                        crossfades.apply(&mut animation_state_data);
                    }
                    let mut controller = SkeletonController::new(
                        skeleton_data.clone(),
                        Arc::new(animation_state_data),
                    )
                    .with_settings(
                        SkeletonControllerSettings::new()
                            .with_cull_direction(CullDirection::CounterClockwise)
                            .with_premultiplied_alpha(skeleton_data_asset.premultiplied_alpha),
                    );
                    let events = spine_event_queue.0.clone();
                    controller
                        .animation_state
                        .set_listener(move |_, animation_event| match animation_event {
                            AnimationEvent::Start { track_entry } => {
                                let mut events = events.lock().unwrap();
                                events.push_back(SpineEvent::Start {
                                    entity: spine_entity,
                                    animation: track_entry.animation().name().to_owned(),
                                });
                            }
                            AnimationEvent::Interrupt { track_entry } => {
                                let mut events = events.lock().unwrap();
                                events.push_back(SpineEvent::Interrupt {
                                    entity: spine_entity,
                                    animation: track_entry.animation().name().to_owned(),
                                });
                            }
                            AnimationEvent::End { track_entry } => {
                                let mut events = events.lock().unwrap();
                                events.push_back(SpineEvent::End {
                                    entity: spine_entity,
                                    animation: track_entry.animation().name().to_owned(),
                                });
                            }
                            AnimationEvent::Complete { track_entry } => {
                                let mut events = events.lock().unwrap();
                                events.push_back(SpineEvent::Complete {
                                    entity: spine_entity,
                                    animation: track_entry.animation().name().to_owned(),
                                });
                            }
                            AnimationEvent::Dispose { .. } => {
                                let mut events = events.lock().unwrap();
                                events.push_back(SpineEvent::Dispose {
                                    entity: spine_entity,
                                });
                            }
                            AnimationEvent::Event {
                                name,
                                int,
                                float,
                                string,
                                audio_path,
                                volume,
                                balance,
                                ..
                            } => {
                                let mut events = events.lock().unwrap();
                                events.push_back(SpineEvent::Event {
                                    entity: spine_entity,
                                    name: name.to_owned(),
                                    int,
                                    float,
                                    string: string.to_owned(),
                                    audio_path: audio_path.to_owned(),
                                    volume,
                                    balance,
                                });
                            }
                        });
                    controller.skeleton.set_to_setup_pose();
                    let mut bones = HashMap::new();
                    if let Some(mut entity_commands) = commands.get_entity(spine_entity) {
                        entity_commands
                            .with_children(|parent| {
                                // TODO: currently, a mesh is created for each slot, however when we use the
                                // combined drawer, this many meshes is usually not necessary. instead, we
                                // may want to dynamically create meshes as needed in the render system
                                parent
                                    .spawn((
                                        Name::new("spine_meshes"),
                                        SpineMeshes,
                                        Transform::from_xyz(0., 0., 0.),
                                        GlobalTransform::default(),
                                        Visibility::default(),
                                        InheritedVisibility::default(),
                                        ViewVisibility::default(),
                                    ))
                                    .with_children(|parent| {
                                        let mut z = 0.;
                                        for (index, _) in controller.skeleton.slots().enumerate() {
                                            let mut mesh = Mesh::new(
                                                PrimitiveTopology::TriangleList,
                                                RenderAssetUsages::MAIN_WORLD
                                                    | RenderAssetUsages::RENDER_WORLD,
                                            );
                                            empty_mesh(&mut mesh);
                                            let mesh_handle = meshes.add(mesh);
                                            parent.spawn((
                                                Name::new(format!("spine_mesh {}", index)),
                                                SpineMesh {
                                                    spine_entity,
                                                    handle: mesh_handle.clone(),
                                                    state: SpineMeshState::Empty,
                                                },
                                                Transform::from_xyz(0., 0., z),
                                                GlobalTransform::default(),
                                                Visibility::default(),
                                                InheritedVisibility::default(),
                                                ViewVisibility::default(),
                                            ));
                                            z += 0.001;
                                        }
                                    });
                                if *with_children {
                                    spawn_bones(
                                        spine_entity,
                                        None,
                                        parent,
                                        &controller.skeleton,
                                        controller.skeleton.bone_root().handle(),
                                        &mut bones,
                                    );
                                }
                            })
                            .insert(Spine(controller));
                    }
                    *spine_loader = SpineLoader::Ready;
                    ready_events.0.push(SpineReadyEvent {
                        entity: spine_entity,
                        bones,
                    });
                }
                SkeletonDataStatus::Loading => {}
                SkeletonDataStatus::Failed => {
                    *spine_loader = SpineLoader::Failed;
                }
            }
        }
    }
}

fn spawn_bones(
    spine_entity: Entity,
    bone_parent: Option<SpineBoneParent>,
    parent: &mut ChildBuilder,
    skeleton: &Skeleton,
    bone: BoneHandle,
    bones: &mut HashMap<String, Entity>,
) {
    if let Some(bone) = bone.get(skeleton) {
        let mut transform = Transform::default();
        transform.translation.x = bone.applied_x();
        transform.translation.y = bone.applied_y();
        transform.translation.z = 0.;
        transform.rotation = Quat::from_axis_angle(Vec3::Z, bone.applied_rotation().to_radians());
        transform.scale.x = bone.applied_scale_x();
        transform.scale.y = bone.applied_scale_y();
        let bone_entity = parent
            .spawn((
                Name::new(format!("spine_bone ({})", bone.data().name())),
                transform,
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ))
            .insert(SpineBone {
                spine_entity,
                handle: bone.handle(),
                name: bone.data().name().to_owned(),
                parent: bone_parent,
            })
            .with_children(|parent| {
                for child in bone.children() {
                    spawn_bones(
                        spine_entity,
                        Some(SpineBoneParent {
                            entity: parent.parent_entity(),
                            handle: bone.handle(),
                        }),
                        parent,
                        skeleton,
                        child.handle(),
                        bones,
                    );
                }
            })
            .id();
        bones.insert(bone.data().name().to_owned(), bone_entity);
    }
}

fn spine_ready(
    mut ready_events: ResMut<SpineReadyEvents>,
    mut ready_writer: EventWriter<SpineReadyEvent>,
) {
    for event in take(&mut ready_events.0).into_iter() {
        ready_writer.send(event);
    }
}

fn spine_update_animation(
    mut spine_query: Query<(Entity, &mut Spine)>,
    mut spine_events: EventWriter<SpineEvent>,
    time: Res<Time>,
    spine_event_queue: Res<SpineEventQueue>,
) {
    for (_, mut spine) in spine_query.iter_mut() {
        spine.update(time.delta_secs(), Physics::Update);
    }
    {
        let mut events = spine_event_queue.0.lock().unwrap();
        while let Some(event) = events.pop_front() {
            spine_events.send(event);
        }
    }
}

pub enum SkeletonRenderableKind {
    Simple(Vec<SkeletonRenderable>),
    Combined(Vec<SkeletonCombinedRenderable>),
}

#[allow(clippy::type_complexity)]
fn spine_update_meshes(
    mut spine_query: Query<(&mut Spine, Option<&SpineSettings>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mesh_query: Query<(
        Entity,
        &mut SpineMesh,
        &mut Transform,
        Option<&Mesh2d>,
        Option<&Mesh3d>,
    )>,
    mut commands: Commands,
    meshes_query: Query<(&Parent, &Children), With<SpineMeshes>>,
    asset_server: Res<AssetServer>,
) {
    for (meshes_parent, meshes_children) in meshes_query.iter() {
        let Ok((mut spine, spine_mesh_type)) = spine_query.get_mut(meshes_parent.get()) else {
            continue;
        };
        let SpineSettings {
            mesh_type, drawer, ..
        } = spine_mesh_type.cloned().unwrap_or(SpineSettings::default());
        let mut renderables = match drawer {
            SpineDrawer::Combined => {
                SkeletonRenderableKind::Combined(spine.0.combined_renderables())
            }
            SpineDrawer::Separated => SkeletonRenderableKind::Simple(spine.0.renderables()),
            SpineDrawer::None => continue,
        };
        let mut z = 0.;
        let mut renderable_index = 0;
        for child in meshes_children.iter() {
            if let Ok((
                spine_mesh_entity,
                mut spine_mesh,
                mut spine_mesh_transform,
                spine_2d_mesh,
                spine_3d_mesh,
            )) = mesh_query.get_mut(*child)
            {
                macro_rules! apply_mesh {
                    ($mesh:ident, $condition:expr, $attach:expr, $deattach:ty) => {
                        if $condition {
                            if !$mesh.is_some() {
                                if let Some(mut entity) = commands.get_entity(spine_mesh_entity) {
                                    entity.insert($attach);
                                }
                            }
                        } else {
                            if $mesh.is_some() {
                                if let Some(mut entity) = commands.get_entity(spine_mesh_entity) {
                                    entity.remove::<$deattach>();
                                }
                            }
                        }
                    };
                }
                apply_mesh!(
                    spine_2d_mesh,
                    mesh_type == SpineMeshType::Mesh2D,
                    Mesh2d(spine_mesh.handle.clone()),
                    Mesh2d
                );
                apply_mesh!(
                    spine_3d_mesh,
                    mesh_type == SpineMeshType::Mesh3D,
                    Mesh3d(spine_mesh.handle.clone()),
                    Mesh3d
                );
                let Some(mesh) = meshes.get_mut(&spine_mesh.handle) else {
                    continue;
                };
                let mut empty = true;
                'render: {
                    let (
                        slot_index,
                        attachment_renderer_object,
                        vertices,
                        indices,
                        uvs,
                        colors,
                        dark_colors,
                        blend_mode,
                        premultiplied_alpha,
                    ) = match &mut renderables {
                        SkeletonRenderableKind::Simple(vec) => {
                            let Some(renderable) = vec.get_mut(renderable_index) else {
                                break 'render;
                            };
                            let colors = vec![
                                [
                                    renderable.color.r,
                                    renderable.color.g,
                                    renderable.color.b,
                                    renderable.color.a
                                ];
                                renderable.vertices.len()
                            ];
                            let dark_colors = vec![
                                [
                                    renderable.dark_color.r,
                                    renderable.dark_color.g,
                                    renderable.dark_color.b,
                                    renderable.dark_color.a
                                ];
                                renderable.vertices.len()
                            ];
                            (
                                Some(renderable.slot_index),
                                renderable.attachment_renderer_object,
                                take(&mut renderable.vertices),
                                take(&mut renderable.indices),
                                take(&mut renderable.uvs),
                                colors,
                                dark_colors,
                                renderable.blend_mode,
                                renderable.premultiplied_alpha,
                            )
                        }
                        SkeletonRenderableKind::Combined(vec) => {
                            let Some(renderable) = vec.get_mut(renderable_index) else {
                                break 'render;
                            };
                            (
                                None,
                                renderable.attachment_renderer_object,
                                take(&mut renderable.vertices),
                                take(&mut renderable.indices),
                                take(&mut renderable.uvs),
                                take(&mut renderable.colors),
                                take(&mut renderable.dark_colors),
                                renderable.blend_mode,
                                renderable.premultiplied_alpha,
                            )
                        }
                    };
                    let Some(attachment_render_object) = attachment_renderer_object else {
                        break 'render;
                    };
                    let spine_texture =
                        unsafe { &mut *(attachment_render_object as *mut SpineTexture) };
                    let texture_path = spine_texture.0.clone();
                    let mut normals = vec![];
                    for _ in 0..vertices.len() {
                        normals.push([0., 0., 0.]);
                    }
                    mesh.insert_indices(Indices::U16(indices));
                    mesh.insert_attribute(
                        MeshVertexAttribute::new("Vertex_Position", 0, VertexFormat::Float32x2),
                        vertices,
                    );
                    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
                    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
                    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
                    mesh.insert_attribute(DARK_COLOR_ATTRIBUTE, dark_colors);
                    spine_mesh.state = SpineMeshState::Renderable {
                        info: SpineMaterialInfo {
                            slot_index,
                            texture: asset_server.load(texture_path),
                            blend_mode,
                            premultiplied_alpha,
                        },
                    };
                    spine_mesh_transform.translation.z = z;
                    z += 0.001;
                    empty = false;
                }
                if empty {
                    spine_mesh.state = SpineMeshState::Empty;
                    empty_mesh(mesh);
                }
                renderable_index += 1;
            }
        }
    }
}

fn empty_mesh(mesh: &mut Mesh) {
    let positions: Vec<[f32; 3]> = vec![];
    let normals: Vec<[f32; 3]> = vec![];
    let uvs: Vec<[f32; 2]> = vec![];
    let colors: Vec<[f32; 4]> = vec![];
    let dark_colors: Vec<[f32; 4]> = vec![];

    mesh.remove_indices();
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_attribute(DARK_COLOR_ATTRIBUTE, dark_colors);
}

#[derive(Default)]
struct FixSpineTextures {
    handles: Vec<(Handle<Image>, SpineTextureConfig)>,
}

/// Adjusts Spine textures to render properly.
fn adjust_spine_textures(
    mut local: Local<FixSpineTextures>,
    mut spine_texture_create_events: EventReader<SpineTextureCreateEvent>,
    mut images: ResMut<Assets<Image>>,
) {
    for spine_texture_create_event in spine_texture_create_events.read() {
        local.handles.push((
            spine_texture_create_event.handle.clone(),
            spine_texture_create_event.config,
        ));
    }
    let mut removed_handles = vec![];
    for (handle_index, (handle, handle_config)) in local.handles.iter().enumerate() {
        if let Some(image) = images.get_mut(handle) {
            fn convert_filter(filter: AtlasFilter) -> ImageFilterMode {
                match filter {
                    AtlasFilter::Nearest => ImageFilterMode::Nearest,
                    AtlasFilter::Linear => ImageFilterMode::Linear,
                    _ => {
                        warn!("Unsupported Spine filter: {:?}", filter);
                        ImageFilterMode::Nearest
                    }
                }
            }
            fn convert_wrap(wrap: AtlasWrap) -> ImageAddressMode {
                match wrap {
                    AtlasWrap::ClampToEdge => ImageAddressMode::ClampToEdge,
                    AtlasWrap::MirroredRepeat => ImageAddressMode::MirrorRepeat,
                    AtlasWrap::Repeat => ImageAddressMode::Repeat,
                    _ => {
                        warn!("Unsupported Spine wrap mode: {:?}", wrap);
                        ImageAddressMode::ClampToEdge
                    }
                }
            }
            image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                min_filter: convert_filter(handle_config.min_filter),
                mag_filter: convert_filter(handle_config.mag_filter),
                address_mode_u: convert_wrap(handle_config.u_wrap),
                address_mode_v: convert_wrap(handle_config.v_wrap),
                ..Default::default()
            });
            // The RGB components exported from Spine were premultiplied in nonlinear space, but need to be
            // multiplied in linear space to render properly in Bevy.
            if handle_config.premultiplied_alpha {
                for i in 0..(image.data.len() / 4) {
                    let mut rgba = Srgba::rgba_u8(
                        image.data[i * 4],
                        image.data[i * 4 + 1],
                        image.data[i * 4 + 2],
                        image.data[i * 4 + 3],
                    );
                    if rgba.alpha != 0. {
                        rgba = Srgba::new(
                            rgba.red / rgba.alpha,
                            rgba.green / rgba.alpha,
                            rgba.blue / rgba.alpha,
                            rgba.alpha,
                        );
                    } else {
                        rgba = Srgba::new(0., 0., 0., 0.);
                    }
                    let mut linear_rgba = LinearRgba::from(rgba);
                    linear_rgba.red *= linear_rgba.alpha;
                    linear_rgba.green *= linear_rgba.alpha;
                    linear_rgba.blue *= linear_rgba.alpha;
                    rgba = Srgba::from(linear_rgba);
                    image.data[i * 4] = (rgba.red * 255.) as u8;
                    image.data[i * 4 + 1] = (rgba.green * 255.) as u8;
                    image.data[i * 4 + 2] = (rgba.blue * 255.) as u8;
                    image.data[i * 4 + 3] = (rgba.alpha * 255.) as u8;
                }
            }
            removed_handles.push(handle_index);
        }
    }
    for removed_handle in removed_handles.into_iter().rev() {
        local.handles.remove(removed_handle);
    }
}

mod assets;
mod crossfades;
mod entity_sync;
mod handle;

pub mod materials;
pub mod textures;

#[cfg(test)]
mod test;

#[doc(hidden)]
pub mod prelude {
    pub use crate::{
        Crossfades, SkeletonController, SkeletonData, SkeletonDataHandle, Spine, SpineBone,
        SpineBundle, SpineEvent, SpineLoader, SpineMesh, SpineMeshState, SpinePlugin,
        SpineReadyEvent, SpineSet, SpineSettings, SpineSync, SpineSyncSet, SpineSyncSystem,
        SpineSystem,
    };
    pub use rusty_spine::{BoneHandle, SlotHandle};
}
