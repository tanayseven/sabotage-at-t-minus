use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_rapier2d::prelude::*;

use crate::config::{
    GROUND_PROBE, JUMP_SPEED, PLAYER_ART_ANCHOR, PLAYER_FRAME_SIZE, PLAYER_HEIGHT, PLAYER_SPEED,
    PLAYER_WIDTH,
};
use crate::player_animation::PlayerAnimation;

/// Where the player drops in at the start of a level run.
pub const SPAWN_POSITION: Vec2 = Vec2::new(-380.0, 260.0);
const GROUND_PROBE_LENGTH: f32 = PLAYER_HEIGHT / 2.0 + GROUND_PROBE;

#[derive(Component)]
pub struct Player;

/// Whether the player is standing on something, refreshed once a frame so that
/// jumping and the animation agree on it without each casting its own ray.
#[derive(Component, Default)]
pub struct Grounded(pub bool);

/// The same character is used on the launch pad and in the level, so the
/// caller supplies both the drop point and the marker that owns the entity.
pub fn spawn_player(
    commands: &mut Commands,
    assets: &AssetServer,
    position: Vec2,
    marker: impl Bundle,
) {
    let animation = PlayerAnimation::load(assets);

    commands.spawn((
        marker,
        Player,
        Sprite {
            image: animation.frame(),
            custom_size: Some(Vec2::splat(PLAYER_FRAME_SIZE)),
            ..default()
        },
        // Lifts the art off the entity's origin so the figure stands on the
        // bottom of its collider rather than straddling the middle of it.
        Anchor(Vec2::new(0.0, PLAYER_ART_ANCHOR)),
        animation,
        Transform::from_xyz(position.x, position.y, 0.0),
        RigidBody::Dynamic,
        Collider::cuboid(PLAYER_WIDTH / 2.0, PLAYER_HEIGHT / 2.0),
        LockedAxes::ROTATION_LOCKED,
        Velocity::zero(),
        Grounded::default(),
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

/// Casts a short ray straight down from the player and records what it finds.
pub fn probe_ground(
    rapier: ReadRapierContext,
    mut players: Query<(Entity, &Transform, &mut Grounded), With<Player>>,
) {
    let Ok(context) = rapier.single() else {
        return;
    };

    for (entity, transform, mut grounded) in &mut players {
        grounded.0 = context
            .cast_ray(
                transform.translation.truncate(),
                Vec2::NEG_Y,
                GROUND_PROBE_LENGTH,
                true,
                QueryFilter::default().exclude_collider(entity),
            )
            .is_some();
    }
}

pub fn jump(
    keys: Res<ButtonInput<KeyCode>>,
    mut players: Query<(&Grounded, &mut Velocity), With<Player>>,
) {
    if !keys.any_just_pressed([KeyCode::Space, KeyCode::KeyW]) {
        return;
    }

    for (grounded, mut velocity) in &mut players {
        if grounded.0 {
            velocity.linear.y = JUMP_SPEED;
        }
    }
}
