use bevy::{prelude::*, window::PrimaryWindow};
use bevy_spine::{
    SkeletonController, SkeletonData, SkeletonDataHandle, Spine, SpineBone, SpinePlugin,
    SpineReadyEvent, SpineSet, SpineSync, SpineSyncSet,
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
    commands.spawn(Camera2d);

    let skeleton = SkeletonData::new_from_json(
        asset_server.load("spineboy/export/spineboy-pro.json"),
        asset_server.load("spineboy/export/spineboy-pma.atlas"),
    );
    let skeleton_handle = skeletons.add(skeleton);

    commands.spawn((
        SkeletonDataHandle(skeleton_handle.clone()),
        Transform::from_xyz(-200., -200., 0.).with_scale(Vec3::splat(0.5)),
        SpineSync,
    ));
}

fn on_spawn(
    mut spine_ready_event: MessageReader<SpineReadyEvent>,
    mut spine_query: Query<&mut Spine>,
    mut commands: Commands,
) {
    for event in spine_ready_event.read() {
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
                .and_then(|crosshair_entity| commands.get_entity(*crosshair_entity).ok())
            {
                crosshair_entity.insert(Crosshair);
            }
        }
    }
}

fn ik(
    crosshair_query: Single<(&mut Transform, &SpineBone), With<Crosshair>>,
    window_query: Single<&Window, With<PrimaryWindow>>,
    camera_query: Single<(Entity, &Camera)>,
    global_transform_query: Query<&GlobalTransform>,
) {
    let (camera_entity, camera) = camera_query.into_inner();
    let camera_global_transform = global_transform_query.get(camera_entity).unwrap();
    let window = window_query.into_inner();
    let cursor_position = window
        .cursor_position()
        .and_then(|cursor| {
            camera
                .viewport_to_world(camera_global_transform, cursor)
                .ok()
        })
        .map(|ray| ray.origin.truncate())
        .unwrap_or(Vec2::ZERO);

    let (mut crosshair_transform, crosshair_bone) = crosshair_query.into_inner();
    let parent_global_transform = global_transform_query
        .get(crosshair_bone.parent.as_ref().unwrap().entity)
        .unwrap();
    crosshair_transform.translation = (parent_global_transform.to_matrix().inverse()
        * Vec4::new(cursor_position.x, cursor_position.y, 0., 1.))
    .truncate();
}
