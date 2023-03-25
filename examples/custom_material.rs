use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::MeshVertexBufferLayout,
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
            VertexAttribute, VertexFormat,
        },
    },
    sprite::{Material2d, Material2dKey, Material2dPlugin},
};
use bevy_spine::{
    SkeletonController, SkeletonData, Spine, SpineBundle, SpineDefaultMaterials, SpineMesh,
    SpineMeshState, SpinePlugin, SpineReadyEvent, SpineSet,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(SpinePlugin)
        .add_plugin(Material2dPlugin::<MyMaterial>::default())
        .add_startup_system(setup)
        .add_system(on_spawn.in_set(SpineSet::OnReady))
        .add_system(update_materials)
        .run();
}

fn setup(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut skeletons: ResMut<Assets<SkeletonData>>,
) {
    commands.spawn(Camera2dBundle::default());

    let skeleton = SkeletonData::new_from_json(
        asset_server.load("spineboy/export/spineboy-pro.json"),
        asset_server.load("spineboy/export/spineboy.atlas"),
    );
    let skeleton_handle = skeletons.add(skeleton);

    // Spine with no custom materials
    commands.spawn((SpineBundle {
        skeleton: skeleton_handle.clone(),
        transform: Transform::from_xyz(-230., -130., 0.).with_scale(Vec3::ONE * 0.375),
        ..Default::default()
    },));

    // Spine with custom materials
    commands.spawn((
        SpineBundle {
            skeleton: skeleton_handle.clone(),
            transform: Transform::from_xyz(230., -130., 0.).with_scale(Vec3::ONE * 0.375),
            default_materials: SpineDefaultMaterials::Disabled,
            ..Default::default()
        },
        MySpine,
    ));
}

fn on_spawn(
    mut spine_ready_event: EventReader<SpineReadyEvent>,
    mut spine_query: Query<&mut Spine>,
) {
    for event in spine_ready_event.iter() {
        if let Ok(mut spine) = spine_query.get_mut(event.entity) {
            let Spine(SkeletonController {
                animation_state, ..
            }) = spine.as_mut();
            let _ = animation_state.set_animation_by_name(0, "portal", true);
        }
    }
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn update_materials(
    mut commands: Commands,
    mut spine_query: Query<&Children, (With<Spine>, With<MySpine>)>,
    mut materials: ResMut<Assets<MyMaterial>>,
    mesh_query: Query<(Entity, &SpineMesh, Option<&Handle<MyMaterial>>)>,
) {
    for spine_children in spine_query.iter_mut() {
        for child in spine_children.iter() {
            if let Ok((mesh_entity, spine_mesh, material_handle)) = mesh_query.get(*child) {
                let SpineMeshState::Renderable { texture, .. } = spine_mesh.state.clone() else {
                    continue;
                };
                let handle = if let Some(handle) = material_handle {
                    handle.clone()
                } else {
                    let handle = materials.add(MyMaterial::new(texture.clone()));
                    if let Some(mut entity_commands) = commands.get_entity(mesh_entity) {
                        entity_commands.insert(handle.clone());
                    }
                    handle
                };
                if let Some(material) = materials.get_mut(&handle) {
                    material.image = texture.clone();
                }
            }
        }
    }
}

#[derive(Component)]
pub struct MySpine;

#[derive(AsBindGroup, TypeUuid, Clone)]
#[uuid = "2e85f9ae-049a-4bb5-9f5d-ebaaa208df60"]
pub struct MyMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub image: Handle<Image>,
}

impl MyMaterial {
    pub fn new(image: Handle<Image>) -> Self {
        Self { image }
    }
}

impl Material2d for MyMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/custom_vertex.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/custom_fragment.wgsl".into()
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.vertex.buffers[0]
            .attributes
            .push(VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: 44,
                shader_location: 5,
            });
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}
