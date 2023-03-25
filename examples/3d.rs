use bevy::prelude::*;
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
use bevy_spine::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(SpinePlugin)
        .add_plugin(FlyCameraPlugin)
        .add_startup_system(setup)
        .add_system(on_spawn.in_set(SpineSet::OnReady))
        .add_system(update_materials.in_set(SpineSet::OnUpdateMaterials))
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
        FlyCamera::default(),
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
