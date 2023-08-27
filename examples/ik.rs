use bevy::{prelude::*, window::PrimaryWindow};
use bevy_spine::{
    SkeletonController, SkeletonData, Spine, SpineBone, SpineBundle, SpinePlugin, SpineReadyEvent,
    SpineSet, SpineSync, SpineSyncSet,
};

#[derive(Component)]
pub struct Crosshair;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, SpinePlugin))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                on_spawn.in_set(SpineSet::OnReady),
                ik.in_set(SpineSyncSet::DuringSync),
            ),
        )
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
        asset_server.load("spineboy/export/spineboy-pma.atlas"),
    );
    let skeleton_handle = skeletons.add(skeleton);

    commands.spawn((
        SpineBundle {
            transform: Transform::from_xyz(-200., -200., 0.).with_scale(Vec3::splat(0.5)),
            skeleton: skeleton_handle.clone(),
            ..Default::default()
        },
        SpineSync,
    ));
}

fn on_spawn(
    mut spine_ready_event: EventReader<SpineReadyEvent>,
    mut spine_query: Query<&mut Spine>,
    mut commands: Commands,
) {
    for event in spine_ready_event.iter() {
        if let Ok(mut spine) = spine_query.get_mut(event.entity) {
            let Spine(SkeletonController {
                skeleton,
                animation_state,
                ..
            }) = spine.as_mut();
            skeleton.set_scale(Vec2::splat(1.));
            let _ = animation_state.set_animation_by_name(0, "run", true);
            let _ = animation_state.set_animation_by_name(1, "aim", true);
            let _ = animation_state.set_animation_by_name(2, "shoot", true);
            if let Some(mut crosshair_entity) = event
                .bones
                .get("crosshair")
                .and_then(|crosshair_entity| commands.get_entity(*crosshair_entity))
            {
                crosshair_entity.insert(Crosshair);
            }
        }
    }
}

fn ik(
    mut crosshair_query: Query<(&mut Transform, &SpineBone), With<Crosshair>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(Entity, &Camera)>,
    global_transform_query: Query<&GlobalTransform>,
) {
    let (camera_entity, camera) = camera_query.single();
    let camera_global_transform = global_transform_query.get(camera_entity).unwrap();
    let window = window_query.single();
    let cursor_position = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_global_transform, cursor))
        .map(|ray| ray.origin.truncate())
        .unwrap_or(Vec2::ZERO);

    if let Ok((mut crosshair_transform, crosshair_bone)) = crosshair_query.get_single_mut() {
        let parent_global_transform = global_transform_query
            .get(crosshair_bone.parent.as_ref().unwrap().entity)
            .unwrap();
        crosshair_transform.translation = (parent_global_transform.compute_matrix().inverse()
            * Vec4::new(cursor_position.x, cursor_position.y, 0., 1.))
        .truncate();
    }
}
