use bevy::prelude::*;
use bevy_spine::{
    Crossfades, SkeletonController, SkeletonData, Spine, SpineBundle, SpinePlugin, SpineReadyEvent,
    SpineSet, SpineSystem,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(SpinePlugin)
        .add_startup_system(setup)
        .add_system(on_spawn.in_set(SpineSet::OnReady))
        .add_system(
            crossfades
                .after(SpineSystem::Update)
                .before(SpineSystem::Render),
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
        asset_server.load("spineboy/export/spineboy.atlas"),
    );
    let skeleton_handle = skeletons.add(skeleton);

    let mut crossfades = Crossfades::new();
    crossfades.add("idle", "walk", 0.5);
    crossfades.add("walk", "idle", 0.5);

    commands.spawn(SpineBundle {
        skeleton: skeleton_handle.clone(),
        crossfades,
        transform: Transform::default()
            .with_translation(Vec3::new(0., -200., 0.))
            .with_scale(Vec3::ONE * 0.5),
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
            let _ = animation_state.set_animation_by_name(0, "idle", true);
        }
    }
}

fn crossfades(mut spine_query: Query<&mut Spine>, time: Res<Time>) {
    for mut spine in spine_query.iter_mut() {
        let current_animation = spine
            .animation_state
            .track_at_index(0)
            .unwrap()
            .animation()
            .name()
            .to_owned();
        if time.elapsed_seconds() % 2. > 1. {
            if current_animation != "walk" {
                let _ = spine.animation_state.set_animation_by_name(0, "walk", true);
            }
        } else {
            if current_animation != "idle" {
                let _ = spine.animation_state.set_animation_by_name(0, "idle", true);
            }
        }
    }
}
