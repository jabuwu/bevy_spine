use bevy::{
    ecs::system::StaticSystemParam,
    input::mouse::MouseMotion,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};
use bevy_spine::{
    materials::{SpineMaterial, SpineMaterialInfo, SpineMaterialPlugin, SpineSettingsQuery},
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
            SpineMaterialPlugin::<Spine3DMaterial>::default(),
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
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d::default().mesh().size(5.0, 5.0)),
        material: materials.add(Color::srgb(0.3, 0.5, 0.3)),
        ..default()
    });

    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        Orbit::default(),
    ));

    // spine
    let skeleton = SkeletonData::new_from_json(
        asset_server.load("spineboy/export/spineboy-pro.json"),
        asset_server.load("spineboy/export/spineboy-pma.atlas"),
    );
    let skeleton_handle = skeletons.add(skeleton);
    commands.spawn(SpineBundle {
        skeleton: skeleton_handle.clone(),
        transform: Transform::from_xyz(0., 0., 0.).with_scale(Vec3::ONE * 0.005),
        settings: SpineSettings {
            default_materials: false,
            mesh_type: SpineMeshType::Mesh3D,
            ..Default::default()
        },
        ..Default::default()
    });
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
    let mut window = window_query.single_mut();
    if mouse_buttons.just_pressed(MouseButton::Left) {
        window.cursor.grab_mode = CursorGrabMode::Locked;
        window.cursor.visible = false;
    }
    if keys.just_pressed(KeyCode::Escape) {
        window.cursor.grab_mode = CursorGrabMode::None;
        window.cursor.visible = true;
    }

    let mut mouse_movement = Vec2::ZERO;
    for mouse_motion_event in mouse_motion_events.read() {
        if window.cursor.grab_mode == CursorGrabMode::Locked {
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

#[derive(Component)]
pub struct Spine3DMaterial;

impl SpineMaterial for Spine3DMaterial {
    type Material = StandardMaterial;
    type Params<'w, 's> = SpineSettingsQuery<'w, 's>;

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
                unlit: true,
                alpha_mode: if renderable_data.premultiplied_alpha {
                    AlphaMode::Premultiplied
                } else {
                    AlphaMode::Blend
                },
                ..Self::Material::default()
            });
            material.base_color_texture = Some(renderable_data.texture);
            Some(material)
        } else {
            None
        }
    }
}
