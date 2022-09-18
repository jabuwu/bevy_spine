use bevy::prelude::*;
use bevy_spine::{Spine, SpineBundle, SpinePlugin, SpineReadyEvent, SpineSystem};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(SpinePlugin)
        .add_startup_system(setup)
        .add_system(on_spawn.after(SpineSystem::Load))
        .add_system(ik.after(SpineSystem::Update).before(SpineSystem::Render))
        .run();
}

fn setup(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());

    commands.spawn_bundle(SpineBundle {
        atlas: asset_server.load("spineboy/export/spineboy.atlas"),
        json: asset_server.load("spineboy/export/spineboy-pro.json"),
        transform: Transform::from_xyz(0., 0., 0.).with_scale(Vec3::ONE * 1.),
        ..Default::default()
    });
}

fn on_spawn(
    mut spine_ready_event: EventReader<SpineReadyEvent>,
    mut spine_query: Query<&mut Spine>,
) {
    for event in spine_ready_event.iter() {
        if let Ok(mut spine) = spine_query.get_mut(event.0) {
            if let Some(controller) = spine.controller_mut() {
                let animation_state = &mut controller.animation_state;
                let _ = animation_state.set_animation_by_name(0, "run", true);
                let _ = animation_state.set_animation_by_name(1, "aim", true);
                let _ = animation_state.set_animation_by_name(2, "shoot", true);
            }
        }
    }
}

fn ik(mut spine_query: Query<&mut Spine>, windows: Res<Windows>) {
    for mut spine in spine_query.iter_mut() {
        if let Some(controller) = spine.controller_mut() {
            controller.skeleton.set_scale_x(0.5);
            controller.skeleton.set_scale_y(0.5);
            controller.skeleton.set_y(-200.);
            controller.skeleton.set_x(-200.);
        }
    }
    let cursor_position = if let Some(cursor_position) = windows.primary().cursor_position() {
        cursor_position
    } else {
        return;
    };
    for mut spine in spine_query.iter_mut() {
        if let Some(controller) = spine.controller_mut() {
            let mut bone = if let Some(bone) = controller
                .skeleton
                .bones_mut()
                .find(|bone| bone.data().name() == "crosshair")
            {
                bone
            } else {
                continue;
            };
            let cursor_adjustment = cursor_position
                - Vec2::new(
                    windows.primary().width() * 0.5,
                    windows.primary().height() * 0.5,
                );
            let cursor = bone
                .parent()
                .world_to_local(cursor_adjustment.x, cursor_adjustment.y);
            bone.set_x(cursor.0);
            bone.set_y(cursor.1);
        }
    }
}
