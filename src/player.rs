use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_rapier2d::prelude::*;

use crate::config::{GROUND_PROBE, JUMP_SPEED, PLAYER_SIZE, PLAYER_SPEED};
use crate::tiles::load_pixel_art;

/// Where the player drops in at the start of a level run.
pub const SPAWN_POSITION: Vec2 = Vec2::new(-380.0, 260.0);
const GROUND_PROBE_LENGTH: f32 = PLAYER_SIZE / 2.0 + GROUND_PROBE;

const IDLE_FRAME: &str = "player/character_green_idle.png";
const JUMP_FRAME: &str = "player/character_green_jump.png";
const WALK_FRAMES: [&str; 2] = [
    "player/character_green_walk_a.png",
    "player/character_green_walk_b.png",
];

/// Every frame is a square canvas with the character standing on its bottom
/// edge, filling this much of the canvas height. Sizing to the character rather
/// than the canvas is what keeps the drawn figure the height of the collider.
const CHARACTER_FILL: f32 = 97.0 / 128.0;
const FRAME_SIZE: f32 = PLAYER_SIZE / CHARACTER_FILL;
/// Pins the bottom of the frame — the character's feet — to the underside of
/// the collider, so the empty headroom in the art hangs above it.
const FEET_ANCHOR: Anchor = Anchor(Vec2::new(0.0, PLAYER_SIZE / 2.0 / FRAME_SIZE - 0.5));

/// How long one foot is held down; two of them make a full stride.
const STRIDE_SECONDS: f32 = 0.13;
/// Slower than this counts as standing still. Guards against the residual
/// horizontal drift a physics body has when it is meant to be at rest.
const WALKING_SPEED: f32 = 1.0;

#[derive(Component)]
pub struct Player;

/// The frames to animate from, carried on the entity so the animation system
/// never has to reach for the asset server.
#[derive(Component)]
pub struct PlayerAnimation {
    idle: Handle<Image>,
    jump: Handle<Image>,
    walk: [Handle<Image>; 2],
    stride: Timer,
    foot: usize,
}

/// The same character is used on the launch pad and in the level, so the
/// caller supplies both the drop point and the marker that owns the entity.
pub fn spawn_player(
    commands: &mut Commands,
    assets: &AssetServer,
    position: Vec2,
    marker: impl Bundle,
) {
    let animation = PlayerAnimation {
        idle: load_pixel_art(assets, IDLE_FRAME),
        jump: load_pixel_art(assets, JUMP_FRAME),
        walk: WALK_FRAMES.map(|path| load_pixel_art(assets, path)),
        stride: Timer::from_seconds(STRIDE_SECONDS, TimerMode::Repeating),
        foot: 0,
    };

    commands.spawn((
        marker,
        Player,
        Sprite {
            image: animation.idle.clone(),
            custom_size: Some(Vec2::splat(FRAME_SIZE)),
            ..default()
        },
        FEET_ANCHOR,
        animation,
        Transform::from_xyz(position.x, position.y, 0.0),
        RigidBody::Dynamic,
        Collider::cuboid(PLAYER_SIZE / 2.0, PLAYER_SIZE / 2.0),
        LockedAxes::ROTATION_LOCKED,
        Velocity::zero(),
        Friction::coefficient(0.0),
        Ccd::enabled(),
    ));
}

/// Is there something solid directly under this body? Used both to allow a jump
/// and to decide whether the character is drawn walking or airborne.
fn is_grounded(context: &RapierContext, entity: Entity, transform: &Transform) -> bool {
    context
        .cast_ray(
            transform.translation.truncate(),
            Vec2::NEG_Y,
            GROUND_PROBE_LENGTH,
            true,
            QueryFilter::default().exclude_collider(entity),
        )
        .is_some()
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
        if is_grounded(&context, entity, transform) {
            velocity.linear.y = JUMP_SPEED;
        }
    }
}

/// Picks the frame that matches what the body is actually doing: airborne, and
/// otherwise walking or standing. Driven off velocity rather than the keys, so
/// it stays honest when something else pushes the player around.
pub fn animate_player(
    time: Res<Time>,
    rapier: ReadRapierContext,
    mut players: Query<(
        Entity,
        &Transform,
        &Velocity,
        &mut Sprite,
        &mut PlayerAnimation,
    )>,
) {
    let Ok(context) = rapier.single() else {
        return;
    };

    for (entity, transform, velocity, mut sprite, mut animation) in &mut players {
        let speed = velocity.linear.x;
        let walking = speed.abs() > WALKING_SPEED;

        if walking {
            // Only turn on a real step, so the character keeps facing the way
            // it was heading when it comes to a stop.
            sprite.flip_x = speed < 0.0;
        }

        sprite.image = if !is_grounded(&context, entity, transform) {
            animation.jump.clone()
        } else if walking {
            animation.stride.tick(time.delta());
            if animation.stride.just_finished() {
                animation.foot = 1 - animation.foot;
            }
            animation.walk[animation.foot].clone()
        } else {
            animation.stride.reset();
            animation.foot = 0;
            animation.idle.clone()
        };
    }
}
