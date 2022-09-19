use bevy::prelude::*;
use bevy_spine::{
    SkeletonController, Spine, SpineBundle, SpinePlugin, SpineReadyEvent, SpineSystem,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(SpinePlugin)
        .add_startup_system(setup)
        .add_system(on_spawn.before(SpineSystem::Update))
        .run();
}

fn setup(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());

    commands.spawn_bundle(SpineBundle {
        atlas: asset_server.load("spineboy/export/spineboy.atlas"),
        json: asset_server.load("spineboy/export/spineboy-pro.json"),
        transform: Transform::from_xyz(0., -200., 0.).with_scale(Vec3::ONE * 0.5),
        ..Default::default()
    });
}

fn on_spawn(
    mut spine_ready_event: EventReader<SpineReadyEvent>,
    mut spine_query: Query<&mut Spine>,
) {
    for event in spine_ready_event.iter() {
        if let Ok(mut spine) = spine_query.get_mut(event.0) {
            let Spine(SkeletonController {
                animation_state, ..
            }) = spine.as_mut();
            let _ = animation_state.set_animation_by_name(0, "portal", true);
        }
    }
}
