use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};
use bevy_spine::prelude::*;

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
        .add_plugins(DefaultPlugins)
        .add_plugin(SpinePlugin)
        .add_startup_system(setup)
        .add_system(on_spawn.in_set(SpineSet::OnReady))
        .add_system(update_materials.in_set(SpineSet::OnUpdateMaterials))
        .add_system(controls)
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
        mesh: meshes.add(shape::Plane::from_size(5.0).into()),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
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
        mesh_type: SpineMeshType::Mesh3d,
        ..Default::default()
    });
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
    mut spine_query: Query<&Children, With<Spine>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mesh_query: Query<(Entity, &SpineMesh, Option<&Handle<StandardMaterial>>)>,
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
                    let handle = materials.add(StandardMaterial {
                        unlit: true,
                        alpha_mode: AlphaMode::Premultiplied,
                        ..Default::default()
                    });
                    if let Some(mut entity_commands) = commands.get_entity(mesh_entity) {
                        entity_commands.insert(handle.clone());
                    }
                    handle
                };
                if let Some(material) = materials.get_mut(&handle) {
                    material.base_color_texture = Some(texture.clone());
                }
            }
        }
    }
}

fn controls(
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut orbit_query: Query<(&mut Orbit, &mut Transform)>,
    mouse_buttons: Res<Input<MouseButton>>,
    keys: Res<Input<KeyCode>>,
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
    for mouse_motion_event in mouse_motion_events.iter() {
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
