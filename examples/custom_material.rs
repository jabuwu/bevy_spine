use bevy::{
    ecs::system::{StaticSystemParam, SystemParam},
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
    materials::{SpineMaterial, SpineMaterialPlugin},
    SkeletonController, SkeletonData, Spine, SpineBundle, SpinePlugin, SpineReadyEvent,
    SpineRenderableData, SpineSet, SpineSettings,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(SpinePlugin)
        .add_plugin(Material2dPlugin::<MyMaterial>::default())
        .add_plugin(SpineMaterialPlugin::<MyMaterial>::default())
        .add_startup_system(setup)
        .add_system(on_spawn.in_set(SpineSet::OnReady))
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
            settings: SpineSettings {
                default_materials: false,
                ..Default::default()
            },
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

#[derive(Component)]
pub struct MySpine;

#[derive(AsBindGroup, TypeUuid, Clone, Default)]
#[uuid = "2e85f9ae-049a-4bb5-9f5d-ebaaa208df60"]
pub struct MyMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub image: Handle<Image>,
    #[uniform(2)]
    pub time: f32,
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

#[derive(SystemParam)]
pub struct MyMaterialParam<'w, 's> {
    my_spine_query: Query<'w, 's, &'static MySpine>,
    time: Res<'w, Time>,
}

impl SpineMaterial for MyMaterial {
    type Material = Self;
    type Params<'w, 's> = MyMaterialParam<'w, 's>;

    fn update<'w, 's>(
        material: Option<Self>,
        entity: Entity,
        renderable_data: SpineRenderableData,
        params: &StaticSystemParam<Self::Params<'w, 's>>,
    ) -> Option<Self> {
        if params.my_spine_query.contains(entity) {
            let mut material = material.unwrap_or_else(|| Self::default());
            material.image = renderable_data.texture;
            material.time = params.time.elapsed_seconds();
            Some(material)
        } else {
            None
        }
    }
}
