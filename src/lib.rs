use std::{mem::take, sync::Arc};

use assets::{AtlasLoader, SkeletonJsonLoader};
use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    sprite::Mesh2dHandle,
};
use rusty_spine::{AnimationStateData, CullDirection, SkeletonControllerSettings};

pub use assets::*;
pub use rusty_spine as rusty;
pub use rusty_spine::SkeletonController;

#[derive(Debug)]
pub struct SpineTexture(pub String);

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub enum SpineSystem {
    Load,
    Update,
    Render,
}

pub struct SpinePlugin;

impl Plugin for SpinePlugin {
    fn build(&self, app: &mut App) {
        rusty_spine::extension::set_create_texture_cb(|page, path| {
            page.renderer_object().set(SpineTexture(path.to_owned()));
        });
        rusty_spine::extension::set_dispose_texture_cb(|page| unsafe {
            page.renderer_object().dispose::<SpineTexture>();
        });
        app.add_asset::<Atlas>()
            .add_asset::<SkeletonJson>()
            .init_asset_loader::<AtlasLoader>()
            .init_asset_loader::<SkeletonJsonLoader>()
            .add_event::<SpineReadyEvent>()
            .add_system(spine_load.label(SpineSystem::Load))
            .add_system(
                spine_update
                    .label(SpineSystem::Update)
                    .after(SpineSystem::Load),
            )
            .add_system(
                spine_render
                    .label(SpineSystem::Render)
                    .after(SpineSystem::Update),
            );
    }
}

#[derive(Component)]
pub struct Spine(pub SkeletonController);

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
    pub atlas: Handle<Atlas>,
    pub json: Handle<SkeletonJson>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
}

#[derive(Clone)]
pub struct SpineReadyEvent(pub Entity);

#[derive(Default)]
struct SpineLoadLocal {
    // used for a one-frame delay in sending ready events
    ready: Vec<Entity>,
}

fn spine_load(
    mut skeleton_query: Query<(
        &mut SpineLoader,
        Entity,
        &Handle<Atlas>,
        &Handle<SkeletonJson>,
    )>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut ready_event: EventWriter<SpineReadyEvent>,
    mut local: Local<SpineLoadLocal>,
    atlases: ResMut<Assets<Atlas>>,
    jsons: ResMut<Assets<SkeletonJson>>,
) {
    for entity in local.ready.iter() {
        ready_event.send(SpineReadyEvent(*entity));
    }
    local.ready = vec![];
    for (mut loader, entity, atlas_handle, json_handle) in skeleton_query.iter_mut() {
        if matches!(loader.as_ref(), SpineLoader::Loading) {
            let atlas = if let Some(atlas) = atlases.get(atlas_handle) {
                atlas
            } else {
                continue;
            };
            let json = if let Some(json) = jsons.get(json_handle) {
                json
            } else {
                continue;
            };
            let skeleton_json = rusty_spine::SkeletonJson::new(atlas.atlas.clone());
            let skeleton_data = match skeleton_json.read_skeleton_data(&json.json) {
                Ok(skeleton_data) => Arc::new(skeleton_data),
                Err(_err) => {
                    // TODO: print error?
                    *loader = SpineLoader::Loading;
                    continue;
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
                    for _ in 0..controller.skeleton.slots_count() {
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
                })
                .insert(Spine(controller));
            *loader = SpineLoader::Ready;
            local.ready.push(entity);
        }
    }
}

pub fn spine_update(mut spine_query: Query<&mut Spine>, time: Res<Time>) {
    for mut spine in spine_query.iter_mut() {
        spine.0.update(time.delta_seconds());
    }
}

pub fn spine_render(
    mut spine_query: Query<(&mut Spine, &Children)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    colored_mesh2d: Query<(&Mesh2dHandle, &Handle<ColorMaterial>)>,
    asset_server: Res<AssetServer>,
) {
    for (mut spine, spine_children) in spine_query.iter_mut() {
        spine.0.skeleton.update_world_transform();
        let mut renderables = spine.0.renderables();
        for (renderable_index, child) in spine_children.iter().enumerate() {
            if let Ok((mesh_handle, color_material_handle)) = colored_mesh2d.get(*child) {
                let mesh = meshes.get_mut(&mesh_handle.0).unwrap();
                if let Some(renderable) = renderables.get_mut(renderable_index) {
                    let mut normals = vec![];
                    for _ in 0..renderable.vertices.len() {
                        normals.push([0., 0., 0.]);
                    }
                    mesh.set_indices(Some(Indices::U32(take(&mut renderable.indices))));
                    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, take(&mut renderable.vertices));
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
