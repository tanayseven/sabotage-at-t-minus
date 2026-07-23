//! The player character: its marker component, spawn, and control systems.

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::config::{GROUND_PROBE, JUMP_SPEED, PLAYER_SIZE, PLAYER_SPEED};

#[derive(Component)]
pub struct Player;

/// Spawns the player above the leftmost platform, so the first thing the game
/// shows is the player falling and landing on it.
pub fn spawn_player(commands: &mut Commands) {
    commands.spawn((
        Player,
        Sprite {
            color: Color::srgb(0.9, 0.35, 0.2),
            custom_size: Some(Vec2::splat(PLAYER_SIZE)),
            ..default()
        },
        Transform::from_xyz(-380.0, 260.0, 0.0),
        RigidBody::Dynamic,
        Collider::cuboid(PLAYER_SIZE / 2.0, PLAYER_SIZE / 2.0),
        // Driven by setting velocity directly, so the solver must not tip the
        // player over when it clips a corner.
        LockedAxes::ROTATION_LOCKED,
        Velocity::zero(),
        // Horizontal speed is assigned every frame, so surface friction would
        // only fight the controller.
        Friction::coefficient(0.0),
        // Stops the player tunnelling through a platform at the bottom of a
        // long fall, when per-frame movement can exceed the platform's depth.
        Ccd::enabled(),
    ));
}

/// Steers the player horizontally, leaving the vertical component to gravity
/// and [`jump`]. Writing `Transform` directly would teleport the body through
/// anything in the way.
pub fn move_player(
    keys: Res<ButtonInput<KeyCode>>,
    mut players: Query<&mut Velocity, With<Player>>,
) {
    let mut direction = 0.0;
    if keys.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]) {
        direction -= 1.0;
    }
    if keys.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]) {
        direction += 1.0;
    }

    for mut velocity in &mut players {
        // Assigning rather than accumulating means no input is a full stop:
        // this is a character, not a crate, and it should answer the keyboard
        // directly. `linear.y` is left alone so gravity still applies.
        velocity.linear.x = direction * PLAYER_SPEED;
    }
}

/// Launches the player upwards, but only with something solid underfoot.
///
/// Groundedness is a short ray straight down from the player's centre. The ray
/// excludes the player's own collider, so `solid` casting from inside it is
/// safe.
pub fn jump(
    keys: Res<ButtonInput<KeyCode>>,
    rapier: ReadRapierContext,
    mut players: Query<(Entity, &Transform, &mut Velocity), With<Player>>,
) {
    if !keys.any_just_pressed([KeyCode::Space, KeyCode::KeyW, KeyCode::ArrowUp]) {
        return;
    }

    let Ok(context) = rapier.single() else {
        return;
    };

    for (entity, transform, mut velocity) in &mut players {
        let grounded = context
            .cast_ray(
                transform.translation.truncate(),
                Vec2::NEG_Y,
                PLAYER_SIZE / 2.0 + GROUND_PROBE,
                true,
                QueryFilter::default().exclude_collider(entity),
            )
            .is_some();

        if grounded {
            velocity.linear.y = JUMP_SPEED;
        }
    }
}
