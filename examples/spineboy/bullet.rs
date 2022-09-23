use bevy::prelude::*;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub enum BulletSystem {
    Spawn,
    Update,
}

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<BulletSpawnEvent>()
            .add_system(bullet_spawn.label(BulletSystem::Spawn))
            .add_system(
                bullet_update
                    .label(BulletSystem::Update)
                    .after(BulletSystem::Spawn),
            );
    }
}

pub struct BulletSpawnEvent {
    pub position: Vec2,
    pub velocity: Vec2,
}

#[derive(Component)]
pub struct Bullet {
    pub velocity: Vec2,
}

fn bullet_spawn(mut commands: Commands, mut bullet_spawn_events: EventReader<BulletSpawnEvent>) {
    for event in bullet_spawn_events.iter() {
        commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: Color::RED,
                    custom_size: Some(Vec2::ONE * 16.),
                    ..Default::default()
                },
                transform: Transform::from_translation(event.position.extend(1.)),
                ..Default::default()
            })
            .insert(Bullet {
                velocity: event.velocity,
            });
    }
}

fn bullet_update(mut bullet_query: Query<(&mut Transform, &Bullet)>, time: Res<Time>) {
    for (mut bullet_transform, bullet) in bullet_query.iter_mut() {
        bullet_transform.translation += (bullet.velocity * time.delta_seconds()).extend(0.);
    }
}
