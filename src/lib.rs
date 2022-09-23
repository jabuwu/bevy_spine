use std::{
    collections::VecDeque,
    mem::take,
    sync::{Arc, Mutex},
};

use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, MeshVertexAttribute},
        render_resource::{PrimitiveTopology, VertexFormat},
    },
    sprite::Mesh2dHandle,
};
use rusty::Skeleton;

use crate::{
    assets::{AtlasLoader, SkeletonJsonLoader},
    rusty::{
        draw::CullDirection, AnimationStateData, BoneHandle, EventType, SkeletonControllerSettings,
    },
};

pub use assets::*;
pub use rusty_spine as rusty;
pub use rusty_spine::SkeletonController;

#[derive(Debug)]
pub struct SpineTexture(pub String);

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub enum SpineSystem {
    Load,
    Update,
    SyncEntities,
    SyncBones,
    Render,
}

pub struct SpinePlugin;

impl Plugin for SpinePlugin {
    fn build(&self, app: &mut App) {
        let image_handles: Arc<Mutex<Vec<(String, Handle<Image>)>>> =
            Arc::new(Mutex::new(Vec::new()));
        let image_remember: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let image_forget: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let remember = image_remember.clone();
        rusty_spine::extension::set_create_texture_cb(move |page, path| {
            remember.lock().unwrap().push(path.to_owned());
            page.renderer_object().set(SpineTexture(path.to_owned()));
        });
        let forget = image_forget.clone();
        rusty_spine::extension::set_dispose_texture_cb(move |page| unsafe {
            forget.lock().unwrap().push(
                page.renderer_object()
                    .get_unchecked::<SpineTexture>()
                    .0
                    .clone(),
            );
            page.renderer_object().dispose::<SpineTexture>();
        });
        app.insert_resource(PersistentImageHandles {
            handles: image_handles,
            remember: image_remember,
            forget: image_forget,
        })
        .add_asset::<Atlas>()
        .add_asset::<SkeletonJson>()
        .add_asset::<SkeletonBinary>()
        .add_asset::<SkeletonData>()
        .init_asset_loader::<AtlasLoader>()
        .init_asset_loader::<SkeletonJsonLoader>()
        .init_asset_loader::<SkeletonBinaryLoader>()
        .add_event::<SpineReadyEvent>()
        .add_event::<SpineEvent>()
        .add_system(spine_load.label(SpineSystem::Load))
        .add_system(
            spine_update
                .label(SpineSystem::Update)
                .after(SpineSystem::Load),
        )
        .add_system(
            spine_sync_entities
                .label(SpineSystem::SyncEntities)
                .after(SpineSystem::Update),
        )
        .add_system(
            spine_sync_bones
                .label(SpineSystem::SyncBones)
                .after(SpineSystem::SyncEntities),
        )
        .add_system(
            spine_render
                .label(SpineSystem::Render)
                .after(SpineSystem::SyncBones),
        );
    }
}

#[derive(Component)]
pub struct Spine(pub SkeletonController);

#[derive(Component)]
pub struct SpineBone {
    pub spine_entity: Entity,
    pub handle: BoneHandle,
}

#[derive(Component)]
pub struct SpineMesh;

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

#[derive(Default, Component)]
pub enum SpineLoader {
    #[default]
    Loading,
    Ready,
    Failed,
}

impl SpineLoader {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Default, Bundle)]
pub struct SpineBundle {
    pub loader: SpineLoader,
    pub skeleton: Handle<SkeletonData>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
}

#[derive(Clone)]
pub struct SpineReadyEvent(pub Entity);

#[derive(Clone)]
pub enum SpineEvent {
    Start { entity: Entity, animation: String },
    Interrupt { entity: Entity, animation: String },
    End { entity: Entity, animation: String },
    Complete { entity: Entity, animation: String },
    Dispose { entity: Entity },
    Event { entity: Entity, name: String },
}

#[derive(Default)]
struct SpineLoadLocal {
    // used for a one-frame delay in sending ready events
    ready: Vec<Entity>,
}

struct PersistentImageHandles {
    handles: Arc<Mutex<Vec<(String, Handle<Image>)>>>,
    remember: Arc<Mutex<Vec<String>>>,
    forget: Arc<Mutex<Vec<String>>>,
}

fn spine_load(
    mut skeleton_query: Query<(&mut SpineLoader, Entity, &Handle<SkeletonData>)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut ready_events: EventWriter<SpineReadyEvent>,
    mut local: Local<SpineLoadLocal>,
    mut skeleton_data_assets: ResMut<Assets<SkeletonData>>,
    atlases: ResMut<Assets<Atlas>>,
    jsons: ResMut<Assets<SkeletonJson>>,
    binaries: ResMut<Assets<SkeletonBinary>>,
    persistent_image_handles: Res<PersistentImageHandles>,
    asset_server: Res<AssetServer>,
) {
    for entity in local.ready.iter() {
        ready_events.send(SpineReadyEvent(*entity));
    }
    local.ready = vec![];
    for (mut spine_loader, entity, data_handle) in skeleton_query.iter_mut() {
        if matches!(spine_loader.as_ref(), SpineLoader::Loading) {
            let mut skeleton_data_asset =
                if let Some(skeleton_data_asset) = skeleton_data_assets.get_mut(data_handle) {
                    skeleton_data_asset
                } else {
                    continue;
                };
            let skeleton_data = match &mut skeleton_data_asset {
                SkeletonData::JsonFile {
                    atlas,
                    json,
                    loader,
                    data,
                } => {
                    let atlas = if let Some(atlas) = atlases.get(atlas) {
                        atlas
                    } else {
                        continue;
                    };
                    let json = if let Some(json) = jsons.get(&json) {
                        json
                    } else {
                        continue;
                    };
                    let skeleton_json = if let Some(loader) = &loader {
                        loader
                    } else {
                        *loader = Some(rusty_spine::SkeletonJson::new(atlas.atlas.clone()));
                        loader.as_ref().unwrap()
                    };
                    if let Some(skeleton_data) = &data {
                        skeleton_data.clone()
                    } else {
                        match skeleton_json.read_skeleton_data(&json.json) {
                            Ok(skeleton_data) => {
                                *data = Some(Arc::new(skeleton_data));
                                data.as_ref().unwrap().clone()
                            }
                            Err(_err) => {
                                // TODO: print error?
                                *spine_loader = SpineLoader::Loading;
                                continue;
                            }
                        }
                    }
                }
                SkeletonData::BinaryFile {
                    atlas,
                    binary,
                    loader,
                    data,
                } => {
                    let atlas = if let Some(atlas) = atlases.get(atlas) {
                        atlas
                    } else {
                        continue;
                    };
                    let binary = if let Some(binary) = binaries.get(&binary) {
                        binary
                    } else {
                        continue;
                    };
                    let skeleton_binary = if let Some(loader) = &loader {
                        loader
                    } else {
                        *loader = Some(rusty_spine::SkeletonBinary::new(atlas.atlas.clone()));
                        loader.as_ref().unwrap()
                    };
                    if let Some(skeleton_data) = &data {
                        skeleton_data.clone()
                    } else {
                        match skeleton_binary.read_skeleton_data(&binary.binary) {
                            Ok(skeleton_data) => {
                                *data = Some(Arc::new(skeleton_data));
                                data.as_ref().unwrap().clone()
                            }
                            Err(_err) => {
                                // TODO: print error?
                                *spine_loader = SpineLoader::Loading;
                                continue;
                            }
                        }
                    }
                }
            };
            let animation_state_data = Arc::new(AnimationStateData::new(skeleton_data.clone()));
            let controller = SkeletonController::new(skeleton_data, animation_state_data)
                .with_settings(
                    SkeletonControllerSettings::new()
                        .with_cull_direction(CullDirection::CounterClockwise),
                );
            commands
                .entity(entity)
                .with_children(|parent| {
                    for _ in controller.skeleton.slots() {
                        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
                        empty_mesh(&mut mesh);
                        let mesh_handle = meshes.add(mesh);
                        parent.spawn_bundle((
                            Mesh2dHandle(mesh_handle.clone()),
                            Transform::default(),
                            GlobalTransform::default(),
                            Visibility::default(),
                            ComputedVisibility::default(),
                            materials.add(ColorMaterial {
                                color: Color::NONE,
                                texture: None,
                            }),
                        ));
                    }
                    spawn_bones(
                        entity,
                        parent,
                        &controller.skeleton,
                        controller.skeleton.bone_root().handle(),
                    );
                })
                .insert(Spine(controller));
            *spine_loader = SpineLoader::Ready;
            local.ready.push(entity);
        }
    }
    let mut image_handles = persistent_image_handles.handles.lock().unwrap();
    let mut image_remember = persistent_image_handles.remember.lock().unwrap();
    let mut image_forget = persistent_image_handles.forget.lock().unwrap();
    while let Some(image) = image_remember.pop() {
        image_handles.push((image.clone(), asset_server.load(&image)));
    }
    while let Some(image) = image_forget.pop() {
        if let Some(index) = image_handles.iter().position(|i| i.0 == image) {
            image_handles.remove(index);
        }
    }
}

fn spawn_bones(
    spine_entity: Entity,
    parent: &mut ChildBuilder,
    skeleton: &Skeleton,
    bone: BoneHandle,
) {
    if let Some(bone) = bone.get(skeleton) {
        parent
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::new(8., 32.)),
                    color: Color::NONE,
                    ..Default::default()
                },
                transform: Transform::from_xyz(0., 0., 1.),
                ..Default::default()
            })
            .insert(SpineBone {
                spine_entity,
                handle: bone.handle(),
            })
            .with_children(|parent| {
                for child in bone.children() {
                    spawn_bones(spine_entity, parent, skeleton, child.handle());
                }
            });
    }
}

#[derive(Default)]
struct SpineUpdateLocal {
    events: Arc<Mutex<VecDeque<SpineEvent>>>,
}

fn spine_update(
    mut spine_query: Query<(Entity, &mut Spine)>,
    mut spine_ready_events: EventReader<SpineReadyEvent>,
    mut spine_events: EventWriter<SpineEvent>,
    time: Res<Time>,
    local: Local<SpineUpdateLocal>,
) {
    for event in spine_ready_events.iter() {
        if let Ok((entity, mut spine)) = spine_query.get_mut(event.0) {
            let events = local.events.clone();
            spine.animation_state.set_listener(
                move |_animation_state, event_type, track_entry, spine_event| match event_type {
                    EventType::Start => {
                        let mut events = events.lock().unwrap();
                        events.push_back(SpineEvent::Start {
                            entity,
                            animation: track_entry.animation().name().to_owned(),
                        });
                    }
                    EventType::Interrupt => {
                        let mut events = events.lock().unwrap();
                        events.push_back(SpineEvent::Interrupt {
                            entity,
                            animation: track_entry.animation().name().to_owned(),
                        });
                    }
                    EventType::End => {
                        let mut events = events.lock().unwrap();
                        events.push_back(SpineEvent::End {
                            entity,
                            animation: track_entry.animation().name().to_owned(),
                        });
                    }
                    EventType::Complete => {
                        let mut events = events.lock().unwrap();
                        events.push_back(SpineEvent::Complete {
                            entity,
                            animation: track_entry.animation().name().to_owned(),
                        });
                    }
                    EventType::Dispose => {
                        let mut events = events.lock().unwrap();
                        events.push_back(SpineEvent::Dispose { entity });
                    }
                    EventType::Event => {
                        if let Some(spine_event) = spine_event {
                            let mut events = events.lock().unwrap();
                            events.push_back(SpineEvent::Event {
                                entity,
                                name: spine_event.data().name().to_owned(),
                            });
                        }
                    }
                    _ => {}
                },
            );
        }
    }
    for (_, mut spine) in spine_query.iter_mut() {
        spine.update(time.delta_seconds());
    }
    {
        let mut events = local.events.lock().unwrap();
        while let Some(event) = events.pop_front() {
            spine_events.send(event);
        }
    }
}

fn spine_sync_entities(
    mut bone_query: Query<(&mut Transform, &SpineBone)>,
    spine_query: Query<&Spine>,
) {
    for (mut bone_transform, bone) in bone_query.iter_mut() {
        if let Ok(spine) = spine_query.get(bone.spine_entity) {
            if let Some(bone) = bone.handle.get(&spine.skeleton) {
                bone_transform.translation.x = bone.x();
                bone_transform.translation.y = bone.y();
                bone_transform.rotation =
                    Quat::from_axis_angle(Vec3::Z, bone.rotation().to_radians());
                bone_transform.scale.x = bone.scale_x();
                bone_transform.scale.y = bone.scale_y();
            }
        }
    }
}

fn spine_sync_bones(
    mut bone_query: Query<(&mut Transform, &SpineBone)>,
    mut spine_query: Query<&mut Spine>,
) {
    for (bone_transform, bone) in bone_query.iter_mut() {
        if let Ok(mut spine) = spine_query.get_mut(bone.spine_entity) {
            if let Some(mut bone) = bone.handle.get_mut(&mut spine.skeleton) {
                bone.set_x(bone_transform.translation.x);
                bone.set_y(bone_transform.translation.y);
                let ang = bone_transform.rotation * Vec3::X;
                bone.set_rotation(ang.y.atan2(ang.x).to_degrees());
                bone.set_scale_x(bone_transform.scale.x);
                bone.set_scale_y(bone_transform.scale.y);
            }
        }
    }
    for mut spine in spine_query.iter_mut() {
        spine.0.skeleton.update_world_transform();
    }
    for (mut bone_transform, bone) in bone_query.iter_mut() {
        if let Ok(spine) = spine_query.get(bone.spine_entity) {
            if let Some(bone) = bone.handle.get(&spine.skeleton) {
                bone_transform.translation.x = bone.applied_x();
                bone_transform.translation.y = bone.applied_y();
                bone_transform.rotation =
                    Quat::from_axis_angle(Vec3::Z, bone.applied_rotation().to_radians());
                bone_transform.scale.x = bone.applied_scale_x();
                bone_transform.scale.y = bone.applied_scale_y();
            }
        }
    }
}

fn spine_render(
    mut spine_query: Query<(&mut Spine, &Children, &Transform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    colored_mesh2d: Query<(&Mesh2dHandle, &Handle<ColorMaterial>)>,
    asset_server: Res<AssetServer>,
) {
    for (mut spine, spine_children, spine_transform) in spine_query.iter_mut() {
        spine.0.settings.cull_direction = CullDirection::CounterClockwise;
        if spine_transform.scale.x < 0. {
            if spine_transform.scale.y > 0. {
                spine.0.settings.cull_direction = CullDirection::Clockwise;
            }
        }
        if spine_transform.scale.y < 0. {
            if spine_transform.scale.y > 0. {
                spine.0.settings.cull_direction = CullDirection::Clockwise;
            }
        }
        let mut renderables = spine.0.renderables();
        for (renderable_index, child) in spine_children.iter().enumerate() {
            if let Ok((mesh_handle, color_material_handle)) = colored_mesh2d.get(*child) {
                let mesh = meshes.get_mut(&mesh_handle.0).unwrap();
                if let Some(renderable) = renderables.get_mut(renderable_index) {
                    let mut normals = vec![];
                    for _ in 0..renderable.vertices.len() {
                        normals.push([0., 0., 0.]);
                    }
                    mesh.set_indices(Some(Indices::U16(take(&mut renderable.indices))));
                    mesh.insert_attribute(
                        MeshVertexAttribute::new("Vertex_Position", 0, VertexFormat::Float32x2),
                        take(&mut renderable.vertices),
                    );
                    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
                    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, take(&mut renderable.uvs));
                    if let Some(color_material) = color_materials.get_mut(color_material_handle) {
                        color_material.color.set_r(renderable.color.r);
                        color_material.color.set_g(renderable.color.g);
                        color_material.color.set_b(renderable.color.b);
                        color_material.color.set_a(renderable.color.a);
                        let texture_path = if let Some(attachment_render_object) =
                            renderable.attachment_renderer_object
                        {
                            let spine_texture =
                                unsafe { &mut *(attachment_render_object as *mut SpineTexture) };
                            Some(spine_texture.0.clone())
                        } else {
                            None
                        };
                        color_material.texture =
                            texture_path.map(|p| asset_server.load(p.as_str()));
                    }
                } else {
                    empty_mesh(mesh);
                }
            }
        }
    }
}

fn empty_mesh(mesh: &mut Mesh) {
    let indices = Indices::U32(vec![]);

    let positions: Vec<[f32; 3]> = vec![];
    let normals: Vec<[f32; 3]> = vec![];
    let uvs: Vec<[f32; 2]> = vec![];

    mesh.set_indices(Some(indices));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
}

mod assets;
