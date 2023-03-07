// Converts spine textures to use the nearest filter mode. Necessary for PMA textures to render
// properly.

// See: https://github.com/bevyengine/bevy/issues/6315

use bevy::{
    prelude::*,
    render::{
        render_resource::{FilterMode, SamplerDescriptor},
        texture::ImageSampler,
    },
};
use bevy_spine::{
    textures::SpineTextureCreateEvent, SkeletonController, SkeletonData, Spine, SpineBundle,
    SpinePlugin, SpineReadyEvent, SpineSet, SpineSystem,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(SpinePlugin)
        .add_startup_system(setup)
        .add_system(on_spawn.in_set(SpineSet::OnReady))
        .add_system(set_spine_texture_nearest.after(SpineSystem::Load))
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

    commands.spawn(SpineBundle {
        skeleton: skeleton_handle.clone(),
        transform: Transform::from_xyz(0., -300., 0.).with_scale(Vec3::ONE * 1.0),
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
        }
    }
}

#[derive(Default)]
struct SetSpineTextureNearest {
    handles: Vec<Handle<Image>>,
}

fn set_spine_texture_nearest(
    mut local: Local<SetSpineTextureNearest>,
    mut spine_texture_create_events: EventReader<SpineTextureCreateEvent>,
    mut images: ResMut<Assets<Image>>,
) {
    for spine_texture_create_event in spine_texture_create_events.iter() {
        local
            .handles
            .push(spine_texture_create_event.handle.clone());
    }
    let mut removed_handles = vec![];
    for (handle_index, handle) in local.handles.iter().enumerate() {
        if let Some(image) = images.get_mut(&handle) {
            image.sampler_descriptor = ImageSampler::Descriptor(SamplerDescriptor {
                mag_filter: FilterMode::Nearest,
                min_filter: FilterMode::Nearest,
                ..Default::default()
            });
            removed_handles.push(handle_index);
        }
    }
    for removed_handle in removed_handles.into_iter().rev() {
        local.handles.remove(removed_handle);
    }
}
