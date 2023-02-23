use bevy::prelude::*;
use bevy_spine::{
    SkeletonController, SkeletonData, Spine, SpineBundle, SpinePlugin, SpineReadyEvent, SpineSet,
    SpineSystem,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(SpinePlugin)
        .add_startup_system(setup)
        .add_system(on_spawn.in_set(SpineSet::OnReady))
        .add_system(ik.after(SpineSystem::Update).before(SpineSystem::Render))
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

    commands.spawn(SpineBundle {
        skeleton: skeleton_handle.clone(),
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
            let _ = animation_state.set_animation_by_name(0, "run", true);
            let _ = animation_state.set_animation_by_name(1, "aim", true);
            let _ = animation_state.set_animation_by_name(2, "shoot", true);
        }
    }
}

fn ik(mut spine_query: Query<&mut Spine>, window_query: Query<&Window>) {
    for mut spine in spine_query.iter_mut() {
        let Spine(SkeletonController { skeleton, .. }) = spine.as_mut();
        skeleton.set_scale_x(0.5);
        skeleton.set_scale_y(0.5);
        skeleton.set_y(-200.);
        skeleton.set_x(-200.);
    }
    let cursor_position = if let Some(cursor_position) = window_query.single().cursor_position() {
        cursor_position
    } else {
        return;
    };
    for mut spine in spine_query.iter_mut() {
        let Spine(SkeletonController { skeleton, .. }) = spine.as_mut();
        let mut bone = if let Some(bone) = skeleton
            .bones_mut()
            .find(|bone| bone.data().name() == "crosshair")
        {
            bone
        } else {
            continue;
        };
        let cursor_adjustment = cursor_position
            - Vec2::new(
                window_query.single().width() * 0.5,
                window_query.single().height() * 0.5,
            );
        let cursor = bone
            .parent()
            .unwrap()
            .world_to_local(cursor_adjustment.x, cursor_adjustment.y);
        bone.set_x(cursor.0);
        bone.set_y(cursor.1);
    }
}
