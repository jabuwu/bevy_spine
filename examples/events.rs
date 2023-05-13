use bevy::prelude::*;
use bevy_spine::{
    SkeletonController, SkeletonData, Spine, SpineBundle, SpineEvent, SpinePlugin, SpineReadyEvent,
    SpineSet,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(SpinePlugin)
        .add_startup_system(setup)
        .add_system(on_spawn.in_set(SpineSet::OnReady))
        .add_system(on_spine_event.in_set(SpineSet::OnEvent))
        .add_system(footstep_update)
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
        transform: Transform::from_xyz(0., -200., 0.).with_scale(Vec3::ONE * 0.5),
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
            let _ = animation_state.set_animation_by_name(0, "walk", true);
        }
    }
}

fn on_spine_event(
    mut spine_events: EventReader<SpineEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for event in spine_events.iter() {
        if let SpineEvent::Event { name, .. } = event {
            commands
                .spawn(Text2dBundle {
                    text: Text::from_section(
                        name.as_str(),
                        TextStyle {
                            font: asset_server.load("FiraMono-Medium.ttf"),
                            font_size: 22.0,
                            color: Color::WHITE,
                        },
                    )
                    .with_alignment(TextAlignment::Center),
                    transform: Transform::from_xyz(0., -200., 1.),
                    ..Default::default()
                })
                .insert(Footstep);
        }
    }
}

#[derive(Component)]
struct Footstep;

fn footstep_update(
    mut footstep_query: Query<(&mut Transform, &mut Text, Entity), With<Footstep>>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (mut transform, mut text, entity) in footstep_query.iter_mut() {
        transform.translation.y += time.delta_seconds() * 70.;
        let mut alpha = text.sections[0].style.color.a();
        alpha = (alpha - time.delta_seconds() * 2.).clamp(0., 1.);
        text.sections[0].style.color.set_a(alpha);
        if alpha == 0. {
            commands.entity(entity).despawn();
        }
    }
}
