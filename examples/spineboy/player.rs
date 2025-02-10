use bevy::prelude::*;
use bevy_spine::prelude::*;

use crate::bullet::{BulletSpawnEvent, BulletSystem};

const PLAYER_TRACK_PORTAL: usize = 0;
const PLAYER_TRACK_IDLE: usize = 0;
const PLAYER_TRACK_RUN: usize = 1;
const PLAYER_TRACK_JUMP: usize = 2;
const PLAYER_TRACK_AIM: usize = 3;
const PLAYER_TRACK_SHOOT: usize = 4;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum PlayerSystem {
    Spawn,
    SpineReady,
    SpineEvents,
    Aim,
    Shoot,
    Move,
    Jump,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerSpawnEvent>().add_systems(
            Update,
            (
                player_spawn.in_set(PlayerSystem::Spawn),
                player_spine_ready.in_set(PlayerSystem::SpineReady),
                player_spine_events
                    .in_set(PlayerSystem::SpineEvents)
                    .in_set(SpineSyncSet::BeforeSync),
                player_aim
                    .in_set(PlayerSystem::Aim)
                    .in_set(SpineSyncSet::DuringSync),
                player_shoot
                    .in_set(PlayerSystem::Shoot)
                    .in_set(SpineSyncSet::AfterSync)
                    .before(BulletSystem::Spawn),
                player_move
                    .in_set(PlayerSystem::Move)
                    .in_set(SpineSyncSet::BeforeSync),
                player_jump
                    .in_set(PlayerSystem::Jump)
                    .in_set(SpineSyncSet::BeforeSync),
            ),
        );
    }
}

#[derive(Event)]
pub struct PlayerSpawnEvent {
    pub skeleton: Handle<SkeletonData>,
}

#[derive(Component)]
pub struct Player {
    spawned: bool,
    movement_velocity: f32,
}

#[derive(Component)]
pub struct CrosshairController {
    bone: Entity,
}

#[derive(Component)]
pub struct ShootController {
    cooldown: f32,
    spine: Entity,
    bone: Entity,
}

fn player_spawn(mut commands: Commands, mut player_spawn_events: EventReader<PlayerSpawnEvent>) {
    for event in player_spawn_events.read() {
        commands
            .spawn(SpineBundle {
                skeleton: event.skeleton.clone().into(),
                transform: Transform::from_xyz(-300., -200., 0.).with_scale(Vec3::ONE * 0.25),
                ..Default::default()
            })
            .insert(SpineSync)
            .insert(Player {
                spawned: false,
                movement_velocity: 0.,
            });
    }
}

fn player_spine_ready(
    mut spine_ready_events: EventReader<SpineReadyEvent>,
    mut spine_query: Query<(&mut Spine, Entity), With<Player>>,
    mut spine_bone_query: Query<(&mut SpineBone, Entity)>,
    mut commands: Commands,
) {
    for event in spine_ready_events.read() {
        if let Ok((mut spine, spine_entity)) = spine_query.get_mut(event.entity) {
            let Spine(SkeletonController {
                animation_state,
                skeleton,
                ..
            }) = spine.as_mut();
            let _ = animation_state.set_animation_by_name(PLAYER_TRACK_PORTAL, "portal", false);
            for (bone, bone_entity) in spine_bone_query.iter_mut() {
                if let Some(bone) = bone.handle.get(skeleton) {
                    if bone.data().name() == "crosshair" {
                        commands
                            .entity(spine_entity)
                            .insert(CrosshairController { bone: bone_entity });
                    } else if bone.data().name() == "gun-tip" {
                        commands.entity(spine_entity).insert(ShootController {
                            cooldown: 0.,
                            spine: spine_entity,
                            bone: bone_entity,
                        });
                    }
                }
            }
        }
    }
}

fn player_spine_events(
    mut spine_events: EventReader<SpineEvent>,
    mut spine_query: Query<(&mut Spine, &mut Player)>,
) {
    for event in spine_events.read() {
        if let SpineEvent::Complete { entity, animation } = event {
            if let Ok((mut spine, mut player)) = spine_query.get_mut(*entity) {
                let Spine(controller) = spine.as_mut();
                if animation == "portal" {
                    let _ = controller.animation_state.set_animation_by_name(
                        PLAYER_TRACK_IDLE,
                        "idle",
                        true,
                    );
                    let _ = controller.animation_state.set_animation_by_name(
                        PLAYER_TRACK_AIM,
                        "aim",
                        true,
                    );
                    let mut run_track = controller
                        .animation_state
                        .set_animation_by_name(PLAYER_TRACK_RUN, "run", true)
                        .unwrap();
                    run_track.set_shortest_rotation(true);
                    controller
                        .animation_state
                        .track_at_index_mut(PLAYER_TRACK_AIM)
                        .unwrap()
                        .set_alpha(0.);
                    controller
                        .animation_state
                        .track_at_index_mut(PLAYER_TRACK_RUN)
                        .unwrap()
                        .set_alpha(0.);
                    player.spawned = true;
                } else if animation == "jump" {
                    controller.animation_state.clear_track(PLAYER_TRACK_JUMP);
                }
            }
        }
    }
}

fn player_aim(
    mut crosshair_query: Query<(&mut Spine, Entity, &CrosshairController, &Player)>,
    bone_query: Query<(Entity, &Parent), With<SpineBone>>,
    mut transform_query: Query<&mut Transform>,
    global_transform_query: Query<&GlobalTransform>,
    window_query: Query<&Window>,
    camera_query: Query<(Entity, &Camera)>,
    time: Res<Time>,
) {
    let (camera_entity, camera) = camera_query.single();
    let camera_global_transform = global_transform_query.get(camera_entity).unwrap();
    let Ok(window) = window_query.get_single() else {
        return;
    };
    let cursor_position = window
        .cursor_position()
        .and_then(|cursor| {
            camera
                .viewport_to_world(camera_global_transform, cursor)
                .ok()
        })
        .map(|ray| ray.origin.truncate())
        .unwrap_or(Vec2::ZERO);
    for (mut spine, player_entity, crosshair, player) in crosshair_query.iter_mut() {
        if player.spawned {
            if let Ok((crosshair_entity, crosshair_parent)) = bone_query.get(crosshair.bone) {
                let matrix = if let Ok(parent_transform) =
                    global_transform_query.get(crosshair_parent.get())
                {
                    parent_transform.compute_matrix().inverse()
                } else {
                    Mat4::IDENTITY
                };
                let mut scale_x = 1.;
                if let Ok(mut crosshair_transform) = transform_query.get_mut(crosshair_entity) {
                    crosshair_transform.translation =
                        (matrix * cursor_position.extend(0.).extend(1.)).truncate();
                    if crosshair_transform.translation.x < 0. {
                        scale_x = -1.;
                    }
                }
                if let Ok(mut player_transform) = transform_query.get_mut(player_entity) {
                    player_transform.scale.x = (scale_x * player_transform.scale.x).signum() * 0.25;
                }
                if let Some(mut aim_track) =
                    spine.animation_state.track_at_index_mut(PLAYER_TRACK_AIM)
                {
                    let alpha = aim_track.alpha() * 2.5;
                    aim_track
                        .set_alpha(lerp::Lerp::lerp(alpha, 1., time.delta_secs()).clamp(0., 1.));
                }
            }
        }
    }
}

fn player_shoot(
    mut shoot_query: Query<(&mut ShootController, &Player)>,
    mut spine_query: Query<(&mut Spine, &Transform)>,
    mut bullet_spawn_events: EventWriter<BulletSpawnEvent>,
    global_transform_query: Query<&GlobalTransform>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
) {
    for (mut shoot, player) in shoot_query.iter_mut() {
        shoot.cooldown = (shoot.cooldown - time.delta_secs()).max(0.);
        if mouse_buttons.just_pressed(MouseButton::Left) && player.spawned && shoot.cooldown == 0. {
            let mut scale_x = 1.;
            if let Ok((mut spine, spine_transform)) = spine_query.get_mut(shoot.spine) {
                let _ =
                    spine
                        .animation_state
                        .set_animation_by_name(PLAYER_TRACK_SHOOT, "shoot", false);
                scale_x = spine_transform.scale.x;
            }
            if let Ok(shoot_transform) = global_transform_query.get(shoot.bone) {
                let (_, rotation, translation) = shoot_transform.to_scale_rotation_translation();
                bullet_spawn_events.send(BulletSpawnEvent {
                    position: translation.truncate(),
                    velocity: (rotation * Vec3::X).truncate() * 1000. * scale_x.signum(),
                });
            }
            shoot.cooldown = 0.25;
        }
    }
}

fn player_move(
    mut player_query: Query<(&mut Player, &mut Transform, &mut Spine)>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    for (mut player, mut player_transform, mut player_spine) in player_query.iter_mut() {
        if player.spawned {
            let mut movement = 0.;
            if keys.pressed(KeyCode::KeyA) {
                movement -= 1.;
            }
            if keys.pressed(KeyCode::KeyD) {
                movement += 1.;
            }
            player.movement_velocity =
                (player.movement_velocity + movement * 20. * time.delta_secs()).clamp(-1., 1.);
            if movement == 0. {
                player.movement_velocity *= 0.0001_f32.powf(time.delta_secs());
            }
            player_transform.translation.x += player.movement_velocity * time.delta_secs() * 500.;
            player_transform.translation.x = player_transform.translation.x.clamp(-500., 500.);
            if let Some(mut track) = player_spine
                .animation_state
                .track_at_index_mut(PLAYER_TRACK_RUN)
            {
                track.set_alpha(player.movement_velocity.abs());
            }
        }
    }
}

fn player_jump(mut player_query: Query<(&mut Spine, &Player)>, keys: Res<ButtonInput<KeyCode>>) {
    for (mut spine, player) in player_query.iter_mut() {
        if !player.spawned {
            continue;
        }
        let Spine(SkeletonController {
            animation_state, ..
        }) = spine.as_mut();
        if let Some(mut jump_track) = animation_state.track_at_index_mut(PLAYER_TRACK_JUMP) {
            let progress =
                (jump_track.track_time() / jump_track.animation().duration()).clamp(0., 1.);
            let mix_out_threshold = 0.9;
            let mix_in_threshold = 0.05;
            if progress > mix_out_threshold {
                jump_track
                    .set_alpha(1. - (progress - mix_out_threshold) / (1. - mix_out_threshold));
            } else if progress < mix_in_threshold {
                jump_track.set_alpha((progress / mix_in_threshold).clamp(0., 1.));
            } else {
                jump_track.set_alpha(1.);
            }
        } else if keys.just_pressed(KeyCode::Space) {
            let _ = animation_state.set_animation_by_name(PLAYER_TRACK_JUMP, "jump", false);
            animation_state
                .track_at_index_mut(PLAYER_TRACK_JUMP)
                .unwrap()
                .set_alpha(0.);
        }
    }
}
