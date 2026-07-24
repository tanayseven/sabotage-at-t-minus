use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::config::{GROUND_PROBE, JUMP_SPEED, PLAYER_SIZE, PLAYER_SPEED};
use crate::setup::GameEntity;

const SPAWN_POSITION: Vec2 = Vec2::new(-380.0, 260.0);
const GROUND_PROBE_LENGTH: f32 = PLAYER_SIZE / 2.0 + GROUND_PROBE;

#[derive(Component)]
pub struct Player;

pub fn spawn_player(commands: &mut Commands) {
    commands.spawn((
        GameEntity,
        Player,
        Sprite {
            color: Color::srgb(0.9, 0.35, 0.2),
            custom_size: Some(Vec2::splat(PLAYER_SIZE)),
            ..default()
        },
        Transform::from_xyz(SPAWN_POSITION.x, SPAWN_POSITION.y, 0.0),
        RigidBody::Dynamic,
        Collider::cuboid(PLAYER_SIZE / 2.0, PLAYER_SIZE / 2.0),
        LockedAxes::ROTATION_LOCKED,
        Velocity::zero(),
        Friction::coefficient(0.0),
        Ccd::enabled(),
    ));
}

pub fn move_player(
    keys: Res<ButtonInput<KeyCode>>,
    mut players: Query<&mut Velocity, With<Player>>,
) {
    let mut direction = 0.0;
    if keys.pressed(KeyCode::KeyA) {
        direction -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) {
        direction += 1.0;
    }

    for mut velocity in &mut players {
        velocity.linear.x = direction * PLAYER_SPEED;
    }
}

pub fn jump(
    keys: Res<ButtonInput<KeyCode>>,
    rapier: ReadRapierContext,
    mut players: Query<(Entity, &Transform, &mut Velocity), With<Player>>,
) {
    if !keys.any_just_pressed([KeyCode::Space, KeyCode::KeyW]) {
        return;
    }

    let Ok(context) = rapier.single() else {
        return;
    };

    for (entity, transform, mut velocity) in &mut players {
        let standing_on_something = context
            .cast_ray(
                transform.translation.truncate(),
                Vec2::NEG_Y,
                GROUND_PROBE_LENGTH,
                true,
                QueryFilter::default().exclude_collider(entity),
            )
            .is_some();

        if standing_on_something {
            velocity.linear.y = JUMP_SPEED;
        }
    }
}
