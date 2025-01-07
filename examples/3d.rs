use bevy::{
    ecs::system::StaticSystemParam,
    input::mouse::MouseMotion,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};
use bevy_spine::{
    materials::{Spine3dSettingsQuery, SpineMaterial3d, SpineMaterialInfo, SpineMaterialPlugin3d},
    prelude::*,
    SpineMeshType,
};

#[derive(Component)]
pub struct Orbit {
    angle: f32,
    pitch: f32,
}

impl Default for Orbit {
    fn default() -> Self {
        Self {
            angle: 90.0_f32.to_radians(),
            pitch: 25.0_f32.to_radians(),
        }
    }
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            SpinePlugin,
            SpineMaterialPlugin3d::<Spine3DMaterial>::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (on_spawn.in_set(SpineSet::OnReady), controls))
        .run();
}

fn setup(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut skeletons: ResMut<Assets<SkeletonData>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(5.0, 5.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
    ));

    // light
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            shadow_depth_bias: 0.05,
            shadow_normal_bias: 0.05,
            illuminance: 5_800.0,
            ..default()
        },
        Transform::from_xyz(0.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        Orbit::default(),
    ));

    // spine
    let skeleton = SkeletonData::new_from_json(
        asset_server.load("spineboy/export/spineboy-pro.json"),
        asset_server.load("spineboy/export/spineboy-pma.atlas"),
    );
    let skeleton_handle = skeletons.add(skeleton);
    commands.spawn((SpineBundle {
        loader: SpineLoader {
            skeleton: skeleton_handle.clone(),
            ..Default::default()
        },
        transform: Transform::from_xyz(0., 0., 0.).with_scale(Vec3::ONE * 0.005),
        settings: SpineSettings {
            default_materials: false,
            mesh_type: SpineMeshType::Mesh3D,
            ..Default::default()
        },
        ..Default::default()
    },));
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

fn controls(
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut orbit_query: Query<(&mut Orbit, &mut Transform)>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let window = window_query.get_single_mut();
    if let Ok(mut window) = window {
        if mouse_buttons.just_pressed(MouseButton::Left) {
            window.cursor_options.grab_mode = CursorGrabMode::Locked;
            window.cursor_options.visible = false;
        }
        if keys.just_pressed(KeyCode::Escape) {
            window.cursor_options.grab_mode = CursorGrabMode::None;
            window.cursor_options.visible = true;
        }

        let mut mouse_movement = Vec2::ZERO;
        for mouse_motion_event in mouse_motion_events.read() {
            if window.cursor_options.grab_mode == CursorGrabMode::Locked {
                mouse_movement += mouse_motion_event.delta;
            }
        }
        for (mut orbit, mut orbit_transform) in orbit_query.iter_mut() {
            orbit.angle = (orbit.angle + mouse_movement.x * 0.001).clamp(0.14159, 3.);
            orbit.pitch = (orbit.pitch + mouse_movement.y * 0.001).clamp(0.1, 1.5);
            orbit_transform.translation =
                Vec3::new(orbit.angle.cos(), orbit.pitch.tan(), orbit.angle.sin()).normalize() * 7.;
            orbit_transform.look_at(Vec3::new(0., 1.5, 0.), Vec3::Y);
        }
    }
}

#[derive(Component)]
pub struct Spine3DMaterial;

impl SpineMaterial3d for Spine3DMaterial {
    type Material = StandardMaterial;
    type Params<'w, 's> = Spine3dSettingsQuery<'w, 's>;

    fn update(
        material: Option<Self::Material>,
        entity: Entity,
        renderable_data: SpineMaterialInfo,
        params: &StaticSystemParam<Self::Params<'_, '_>>,
    ) -> Option<Self::Material> {
        let spine_settings = params
            .spine_settings_query
            .get(entity)
            .copied()
            .unwrap_or(SpineSettings::default());
        if spine_settings.mesh_type == SpineMeshType::Mesh3D {
            let mut material = material.unwrap_or_else(|| Self::Material {
                ..Self::Material::default()
            });
            material.base_color = Color::srgba(1.0, 1.0, 1.0, 1.0);
            material.base_color_texture = Some(renderable_data.texture);
            material.alpha_mode = AlphaMode::Blend;
            material.unlit = true;
            Some(material)
        } else {
            None
        }
    }
}
