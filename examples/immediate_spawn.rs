//! Demonstrates how to spawn a [`SpineBundle`] and use it in one frame.

use bevy::{app::AppExit, prelude::*};
use bevy_spine::{SkeletonData, Spine, SpineBundle, SpinePlugin, SpineReadyEvent, SpineSet};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(SpinePlugin)
        .init_resource::<DemoData>()
        .add_startup_system(setup)
        .add_system(spawn.after(SpineSet::Load).before(SpineSet::LoadFlush))
        .add_system(on_spawn.after(SpineSet::Ready).before(SpineSet::Update))
        .add_system(frame_count.in_base_set(CoreSet::First))
        .run();
}

#[derive(Default, Resource)]
struct DemoData {
    frame_count: usize,
    skeleton_handle: Handle<SkeletonData>,
    spawned: bool,
}

fn setup(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut skeletons: ResMut<Assets<SkeletonData>>,
    mut demo_data: ResMut<DemoData>,
) {
    commands.spawn(Camera2dBundle::default());

    let skeleton = SkeletonData::new_from_json(
        asset_server.load("spineboy/export/spineboy-pro.json"),
        asset_server.load("spineboy/export/spineboy.atlas"),
    );
    demo_data.skeleton_handle = skeletons.add(skeleton);
}

fn spawn(
    skeletons: Res<Assets<SkeletonData>>,
    mut demo_data: ResMut<DemoData>,
    mut commands: Commands,
) {
    if !demo_data.spawned {
        if let Some(skeleton) = skeletons.get(&demo_data.skeleton_handle) {
            if skeleton.is_loaded() {
                commands.spawn(SpineBundle {
                    skeleton: demo_data.skeleton_handle.clone(),
                    transform: Transform::from_xyz(0., -200., 0.).with_scale(Vec3::ONE * 0.5),
                    ..Default::default()
                });
                demo_data.spawned = true;
                println!("spawned on frame: {}", demo_data.frame_count);
            }
        }
    }
}

fn on_spawn(
    mut spine_ready_event: EventReader<SpineReadyEvent>,
    mut app_exit: EventWriter<AppExit>,
    spine_query: Query<&Spine>,
    demo_data: Res<DemoData>,
) {
    for event in spine_ready_event.iter() {
        assert!(spine_query.contains(event.entity));
        println!("ready on frame: {}", demo_data.frame_count);
        app_exit.send_default();
    }
}

fn frame_count(mut demo_data: ResMut<DemoData>) {
    demo_data.frame_count += 1;
}
