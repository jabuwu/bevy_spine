//! Demonstrates how to spawn a Spine entity and use it in one frame.

use bevy::diagnostic::FrameCount;
use bevy::prelude::*;
use bevy_spine::{
    SkeletonData, SkeletonDataHandle, Spine, SpinePlugin, SpineReadyEvent, SpineSet, SpineSystem,
};

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum ExampleSet {
    Spawn,
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, SpinePlugin))
        .init_resource::<DemoData>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                spawn.in_set(ExampleSet::Spawn).after(SpineSystem::Load),
                on_spawn.in_set(SpineSet::OnReady),
                ApplyDeferred
                    .after(ExampleSet::Spawn)
                    .before(SpineSystem::Spawn),
            ),
        )
        .run();
}

#[derive(Default, Resource)]
struct DemoData {
    skeleton_handle: Handle<SkeletonData>,
    spawned: bool,
}

fn setup(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut skeletons: ResMut<Assets<SkeletonData>>,
    mut demo_data: ResMut<DemoData>,
) {
    commands.spawn(Camera2d);

    let skeleton = SkeletonData::new_from_json(
        asset_server.load("spineboy/export/spineboy-pro.json"),
        asset_server.load("spineboy/export/spineboy-pma.atlas"),
    );
    demo_data.skeleton_handle = skeletons.add(skeleton);
}

fn spawn(
    skeletons: Res<Assets<SkeletonData>>,
    mut demo_data: ResMut<DemoData>,
    mut commands: Commands,
    frame_count: Res<FrameCount>,
) {
    if !demo_data.spawned {
        if let Some(skeleton) = skeletons.get(&demo_data.skeleton_handle) {
            if skeleton.is_loaded() {
                commands.spawn((
                    SkeletonDataHandle(demo_data.skeleton_handle.clone()),
                    Transform::from_xyz(0., -200., 0.).with_scale(Vec3::ONE * 0.5),
                ));
                demo_data.spawned = true;
                println!("spawned on frame: {}", frame_count.0);
            }
        }
    }
}

fn on_spawn(
    mut spine_ready_event: MessageReader<SpineReadyEvent>,
    mut app_exit: MessageWriter<AppExit>,
    spine_query: Query<&Spine>,
    frame_count: Res<FrameCount>,
) {
    for event in spine_ready_event.read() {
        assert!(spine_query.contains(event.entity));
        println!("ready on frame: {}", frame_count.0);
        app_exit.write_default();
    }
}
