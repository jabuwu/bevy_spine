use std::{ffi::OsStr, path::Path};

use bevy::{prelude::*, render::texture::ImageSettings};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_spine::{
    SkeletonController, SkeletonData, Spine, SpineBundle, SpinePlugin, SpineReadyEvent, SpineSystem,
};
use rfd::FileDialog;

#[cfg(feature = "egui_debugger")]
use rusty_spine::debugger::egui::egui_spine_debugger;

fn main() {
    let mut app = App::new();
    app.insert_resource(ImageSettings::default_nearest())
        .add_plugins(DefaultPlugins)
        .add_plugin(SpinePlugin)
        .add_plugin(EguiPlugin)
        .add_startup_system(setup)
        .add_system(on_spawn.before(SpineSystem::Update))
        // MUST be an exclusive system for RFD to work on MacOS
        .add_system(ui.exclusive_system());
    #[cfg(feature = "egui_debugger")]
    app.add_system(spine_debugger);
    app.run();
}

fn setup(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut skeletons: ResMut<Assets<SkeletonData>>,
) {
    commands.spawn_bundle(Camera2dBundle::default());

    let skeleton = SkeletonData::new_from_json(
        asset_server.load("spineboy/export/spineboy-pro.json"),
        asset_server.load("spineboy/export/spineboy-pma.atlas"),
    );
    let skeleton_handle = skeletons.add(skeleton);

    commands.spawn_bundle(SpineBundle {
        skeleton: skeleton_handle.clone(),
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
            let _ = animation_state.set_animation_by_name(0, "idle", true);
        }
    }
}

fn ui(
    mut egui_context: ResMut<EguiContext>,
    mut spine_query: Query<&mut Spine>,
    mut skeletons: ResMut<Assets<SkeletonData>>,
    mut commands: Commands,
    spine_entity_query: Query<Entity, With<Spine>>,
    asset_server: Res<AssetServer>,
) {
    let mut spine = if let Ok(spine) = spine_query.get_single_mut() {
        spine
    } else {
        return;
    };
    egui::Window::new("Spine Controls").show(egui_context.ctx_mut(), |ui| {
        if ui.button("Open").clicked() {
            let path = std::env::current_dir().unwrap_or(Path::new("/").to_owned());
            let skeleton_file = FileDialog::new()
                .add_filter("Skeleton JSON", &["json"])
                .add_filter("Skeleton Binary", &["skel"])
                .set_directory(path)
                .pick_file();
            if let Some(skeleton_file) = skeleton_file {
                let directory = skeleton_file.parent().unwrap_or(Path::new("/"));
                let atlas_file = FileDialog::new()
                    .add_filter("Atlas", &["atlas"])
                    .set_directory(directory)
                    .pick_file();
                if let Some(atlas_file) = atlas_file {
                    for entity in spine_entity_query.iter() {
                        commands.entity(entity).despawn_recursive();
                    }
                    let skeleton_handle =
                        if skeleton_file.extension().unwrap_or(OsStr::new("")) == "json" {
                            let skeleton = SkeletonData::new_from_json(
                                asset_server.load(skeleton_file),
                                asset_server.load(atlas_file),
                            );
                            skeletons.add(skeleton)
                        } else {
                            let skeleton = SkeletonData::new_from_binary(
                                asset_server.load(skeleton_file),
                                asset_server.load(atlas_file),
                            );
                            skeletons.add(skeleton)
                        };

                    commands.spawn_bundle(SpineBundle {
                        skeleton: skeleton_handle.clone(),
                        transform: Transform::from_xyz(0., -200., 0.).with_scale(Vec3::ONE * 0.5),
                        ..Default::default()
                    });
                }
            }
        }
        ui.collapsing("Animations", |ui| {
            let mut set_animation = None;
            for animation in spine.skeleton.data().animations() {
                if ui.button(animation.name()).clicked() {
                    set_animation = Some(animation.name().to_owned());
                }
            }
            if let Some(animation) = set_animation {
                let _ = spine
                    .animation_state
                    .set_animation_by_name(0, &animation, true);
            }
        });
        ui.collapsing("Tracks", |ui| {
            for track in spine.animation_state.tracks_mut() {
                if let Some(mut track) = track {
                    ui.label(track.animation().name());
                    let mut alpha = track.alpha();
                    ui.add(egui::Slider::new(&mut alpha, 0.0..=1.0));
                    track.set_alpha(alpha);
                } else {
                    ui.label("--");
                }
            }
        });
    });
}

#[cfg(feature = "egui_debugger")]
fn spine_debugger(mut egui_context: ResMut<EguiContext>, mut spine_query: Query<&mut Spine>) {
    for mut spine in spine_query.iter_mut() {
        let Spine(SkeletonController {
            skeleton,
            animation_state,
            ..
        }) = spine.as_mut();
        egui_spine_debugger(egui_context.ctx_mut(), "Spine", skeleton, animation_state);
    }
}
