use bevy::{
    ecs::system::{StaticSystemParam, SystemParam},
    prelude::*,
    reflect::TypePath,
    render::{
        mesh::MeshVertexBufferLayoutRef,
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
        },
    },
    sprite::{Material2d, Material2dKey, Material2dPlugin},
};
use bevy_spine::{
    materials::{
        SpineMaterial, SpineMaterialInfo, SpineMaterialPlugin, DARK_COLOR_ATTRIBUTE,
        DARK_COLOR_SHADER_POSITION,
    },
    SkeletonController, SkeletonData, Spine, SpineBundle, SpineDrawer, SpinePlugin,
    SpineReadyEvent, SpineSet, SpineSettings,
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            SpinePlugin,
            Material2dPlugin::<MyMaterial>::default(),
            SpineMaterialPlugin::<MyMaterial>::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, on_spawn.in_set(SpineSet::OnReady))
        .run();
}

fn setup(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut skeletons: ResMut<Assets<SkeletonData>>,
) {
    commands.spawn(Camera2d);

    let skeleton = SkeletonData::new_from_json(
        asset_server.load("spineboy/export/spineboy-pro.json"),
        asset_server.load("spineboy/export/spineboy.atlas"),
    );
    let skeleton_handle = skeletons.add(skeleton);

    // Spine with no custom materials
    commands.spawn((SpineBundle {
        skeleton: skeleton_handle.clone().into(),
        transform: Transform::from_xyz(-230., -130., 0.).with_scale(Vec3::ONE * 0.375),
        ..Default::default()
    },));

    // Spine with custom materials
    commands.spawn((
        SpineBundle {
            skeleton: skeleton_handle.clone().into(),
            transform: Transform::from_xyz(230., -130., 0.).with_scale(Vec3::ONE * 0.375),
            settings: SpineSettings {
                default_materials: false,
                drawer: SpineDrawer::Separated,
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
    for event in spine_ready_event.read() {
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

#[derive(Asset, AsBindGroup, TypePath, Clone, Default)]
pub struct MyMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub image: Handle<Image>,
    #[uniform(2)]
    pub time: f32,
}

impl Material2d for MyMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/custom.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/custom.wgsl".into()
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_attributes = vec![
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
            Mesh::ATTRIBUTE_COLOR.at_shader_location(4),
            DARK_COLOR_ATTRIBUTE.at_shader_location(DARK_COLOR_SHADER_POSITION as u32),
        ];
        let vertex_buffer_layout = layout.0.get_layout(&vertex_attributes)?;
        descriptor.vertex.buffers = vec![vertex_buffer_layout];
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

#[derive(SystemParam)]
pub struct MyMaterialParam<'w, 's> {
    my_spine_query: Query<'w, 's, &'static Spine, With<MySpine>>,
    time: Res<'w, Time>,
}

impl SpineMaterial for MyMaterial {
    type Material = Self;
    type Params<'w, 's> = MyMaterialParam<'w, 's>;

    fn update(
        material: Option<Self>,
        entity: Entity,
        renderable_data: SpineMaterialInfo,
        params: &StaticSystemParam<Self::Params<'_, '_>>,
    ) -> Option<Self> {
        if let Ok(spine) = params.my_spine_query.get(entity) {
            let mut material = material.unwrap_or_default();
            material.image = renderable_data.texture;
            material.time = params.time.elapsed_secs();
            if let Some(slot) = spine
                .skeleton
                .slot_at_index(renderable_data.slot_index.unwrap_or(9999))
            {
                if slot.data().name().starts_with("portal") {
                    material.time = 0.;
                }
            }
            Some(material)
        } else {
            None
        }
    }
}
