use bevy::prelude::*;
use bevy_spine::{
    SkeletonController, SkeletonData, Spine, SpineBundle, SpineEvent, SpinePlugin, SpineReadyEvent,
    SpineSet,
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, SpinePlugin))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                on_spawn.in_set(SpineSet::OnReady),
                on_spine_event.in_set(SpineSet::OnEvent),
                footstep_update,
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

    commands.spawn(SpineBundle {
        skeleton: skeleton_handle.clone().into(),
        transform: Transform::from_xyz(0., -200., 0.).with_scale(Vec3::ONE * 0.5),
        ..Default::default()
    });
}

fn on_spawn(
    mut spine_ready_event: EventReader<SpineReadyEvent>,
    mut spine_query: Query<&mut Spine>,
) {
    for event in spine_ready_event.read() {
        if let Ok(mut spine) = spine_query.get_mut(event.entity) {
            let Spine(SkeletonController {
                animation_state, ..
            }) = spine.as_mut();
            let _ = animation_state.set_animation_by_name(0, "walk", true);
        }
    }
}

fn on_spine_event(
    mut spine_events: EventReader<SpineEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for event in spine_events.read() {
        if let SpineEvent::Event { name, .. } = event {
            commands
                .spawn((Text2d(name.to_string()), Transform::from_xyz(0., -200., 1.)))
                .insert(Footstep);
        }
    }
}

#[derive(Component)]
struct Footstep;

fn footstep_update(
    mut footstep_query: Query<(&mut Transform, &mut TextColor, Entity), With<Footstep>>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (mut transform, mut text_color, entity) in footstep_query.iter_mut() {
        transform.translation.y += time.delta_secs() * 70.;
        let mut alpha = text_color.alpha();
        alpha = (alpha - time.delta_secs() * 2.).clamp(0., 1.);
        text_color.set_alpha(alpha);
        if alpha == 0. {
            commands.entity(entity).despawn();
        }
    }
}
