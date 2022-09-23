use bevy::prelude::*;
use bevy_spine::{
    SkeletonController, SkeletonData, Spine, SpineBone, SpineBundle, SpineEvent, SpineReadyEvent,
    SpineSystem,
};

use crate::bullet::{BulletSpawnEvent, BulletSystem};

const PLAYER_TRACK_PORTAL: i32 = 0;
const PLAYER_TRACK_IDLE: i32 = 0;
const PLAYER_TRACK_RUN: i32 = 1;
const PLAYER_TRACK_AIM: i32 = 2;
const PLAYER_TRACK_SHOOT: i32 = 3;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub enum PlayerSystem {
    Spawn,
    SpineReady,
    SpineEvents,
    Aim,
    Shoot,
    Move,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerSpawnEvent>()
            .add_system(player_spawn.label(PlayerSystem::Spawn))
            .add_system(player_spine_ready.label(PlayerSystem::SpineReady))
            .add_system(player_spine_events.label(PlayerSystem::SpineEvents))
            .add_system(
                player_aim
                    .label(PlayerSystem::Aim)
                    .after(SpineSystem::SyncEntities)
                    .before(SpineSystem::SyncBones),
            )
            .add_system(
                player_shoot
                    .label(PlayerSystem::Shoot)
                    .after(SpineSystem::SyncBones)
                    .before(BulletSystem::Spawn),
            )
            .add_system(
                player_move
                    .label(PlayerSystem::Move)
                    .after(SpineSystem::Render),
            );
    }
}

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
    for event in player_spawn_events.iter() {
        commands
            .spawn_bundle(SpineBundle {
                skeleton: event.skeleton.clone(),
                transform: Transform::from_xyz(0., -200., 0.).with_scale(Vec3::ONE * 0.25),
                ..Default::default()
            })
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
    for event in spine_ready_events.iter() {
        if let Ok((mut spine, spine_entity)) = spine_query.get_mut(event.0) {
            let Spine(SkeletonController {
                animation_state,
                skeleton,
                ..
            }) = spine.as_mut();
            let _ = animation_state.set_animation_by_name(PLAYER_TRACK_PORTAL, "portal", false);
            for (bone, bone_entity) in spine_bone_query.iter_mut() {
                if let Some(bone) = bone.handle.get(&skeleton) {
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
    for event in spine_events.iter() {
        match event {
            SpineEvent::Complete { entity, animation } => {
                if animation == "portal" {
                    if let Ok((mut spine, mut player)) = spine_query.get_mut(*entity) {
                        let Spine(SkeletonController {
                            animation_state, ..
                        }) = spine.as_mut();
                        let _ =
                            animation_state.set_animation_by_name(PLAYER_TRACK_IDLE, "idle", true);
                        let _ =
                            animation_state.set_animation_by_name(PLAYER_TRACK_AIM, "aim", true);
                        let _ =
                            animation_state.set_animation_by_name(PLAYER_TRACK_RUN, "run", true);
                        player.spawned = true;
                    }
                }
            }
            _ => {}
        }
    }
}

fn player_aim(
    crosshair_query: Query<(&CrosshairController, &Player)>,
    mut bone_query: Query<(&mut Transform, &Parent), With<SpineBone>>,
    global_transform_query: Query<&GlobalTransform>,
    windows: Res<Windows>,
    camera_query: Query<(Entity, &Camera)>,
) {
    let cursor_position = if let Some(cursor_position) = windows.primary().cursor_position() {
        if let Ok((camera_entity, camera)) = camera_query.get_single() {
            if let Ok(camera_transform) = global_transform_query.get(camera_entity) {
                let window_size = Vec2::new(
                    windows.primary().width() as f32,
                    windows.primary().height() as f32,
                );
                let ndc = (cursor_position / window_size) * 2.0 - Vec2::ONE;
                let ndc_to_world =
                    camera_transform.compute_matrix() * camera.projection_matrix().inverse();
                let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));
                world_pos.truncate()
            } else {
                Vec2::ZERO
            }
        } else {
            Vec2::ZERO
        }
    } else {
        Vec2::ZERO
    };
    for (crosshair, player) in crosshair_query.iter() {
        if player.spawned {
            if let Ok((mut crosshair_transform, crosshair_parent)) =
                bone_query.get_mut(crosshair.bone)
            {
                let matrix = if let Ok(parent_transform) =
                    global_transform_query.get(crosshair_parent.get())
                {
                    parent_transform.compute_matrix().inverse()
                } else {
                    Mat4::IDENTITY
                };
                crosshair_transform.translation =
                    (matrix * cursor_position.extend(0.).extend(1.)).truncate();
            }
        }
    }
}

fn player_shoot(
    mut shoot_query: Query<(&mut ShootController, &Player)>,
    mut spine_query: Query<(&mut Spine, &Transform)>,
    mut bullet_spawn_events: EventWriter<BulletSpawnEvent>,
    global_transform_query: Query<&GlobalTransform>,
    mouse_buttons: Res<Input<MouseButton>>,
    time: Res<Time>,
) {
    for (mut shoot, player) in shoot_query.iter_mut() {
        shoot.cooldown = (shoot.cooldown - time.delta_seconds()).max(0.);
        if mouse_buttons.just_pressed(MouseButton::Left) && player.spawned {
            if shoot.cooldown == 0. {
                let mut scale_x = 1.;
                if let Ok((mut spine, spine_transform)) = spine_query.get_mut(shoot.spine) {
                    let _ = spine.animation_state.set_animation_by_name(
                        PLAYER_TRACK_SHOOT,
                        "shoot",
                        false,
                    );
                    scale_x = spine_transform.scale.x;
                }
                if let Ok(shoot_transform) = global_transform_query.get(shoot.bone) {
                    let (_, rotation, translation) =
                        shoot_transform.to_scale_rotation_translation();
                    bullet_spawn_events.send(BulletSpawnEvent {
                        position: translation.truncate(),
                        velocity: (rotation * Vec3::X).truncate() * 1000. * scale_x.signum(),
                    });
                }
                shoot.cooldown = 0.25;
            }
        }
    }
}

fn player_move(
    mut player_query: Query<(&mut Player, &mut Transform, &mut Spine)>,
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    for (mut player, mut player_transform, mut player_spine) in player_query.iter_mut() {
        if player.spawned {
            let mut movement = 0.;
            if keys.pressed(KeyCode::A) {
                movement -= 1.;
            }
            if keys.pressed(KeyCode::D) {
                movement += 1.;
            }
            player.movement_velocity =
                (player.movement_velocity + movement * 20. * time.delta_seconds()).clamp(-1., 1.);
            if movement == 0. {
                player.movement_velocity *= 0.0001_f32.powf(time.delta_seconds());
            }
            if movement > 0. {
                player_transform.scale.x = 0.25;
            } else if movement < 0. {
                player_transform.scale.x = -0.25;
            }
            player_transform.translation.x +=
                player.movement_velocity * time.delta_seconds() * 500.;
            player_transform.translation.x = player_transform.translation.x.clamp(-500., 500.);
            if let Some(mut track) = player_spine
                .animation_state
                .track_at_index_mut(PLAYER_TRACK_RUN as usize)
            {
                track.set_alpha(player.movement_velocity.abs());
            }
        }
    }
}
