use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_rapier2d::prelude::*;

use crate::config::{
    GROUND_PROBE, JUMP_SPEED, PLAYER_ART_ANCHOR, PLAYER_FRAME_SIZE, PLAYER_HEIGHT, PLAYER_SPEED,
    PLAYER_WIDTH,
};
use crate::player_animation::PlayerAnimation;

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
        physics_body(),
    ));
}

/// The body the character is simulated as, kept apart from its art so the
/// physics can be exercised on its own.
fn physics_body() -> impl Bundle {
    (
        RigidBody::Dynamic,
        Collider::cuboid(PLAYER_WIDTH / 2.0, PLAYER_HEIGHT / 2.0),
        LockedAxes::ROTATION_LOCKED,
        Velocity::zero(),
        Grounded::default(),
        // Zero friction on its own is not enough: Rapier averages the two
        // colliders' coefficients by default, so a wall's own friction would
        // still hold the player up while they run into it. Taking the minimum
        // instead keeps the character frictionless against everything, so a
        // wall stops the run without ever slowing the fall.
        Friction {
            coefficient: 0.0,
            combine_rule: CoefficientCombineRule::Min,
        },
        Ccd::enabled(),
    )
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{DESIGN_HEIGHT, GRAVITY, PIXELS_PER_METER, WALL_THICKNESS};
    use crate::physics::configure_physics;
    use std::time::Duration;

    const STEP: f32 = 1.0 / 60.0;
    const STEPS: usize = 30;

    /// How far a body falls from rest under the game's gravity over the whole
    /// run of the simulation below, with nothing to slow it down.
    fn free_fall() -> f32 {
        let seconds = STEP * STEPS as f32;
        0.5 * GRAVITY * seconds * seconds
    }

    /// Holds the player against a wall for a moment and reports how far they
    /// dropped while doing it. Only the physics half of the character is
    /// spawned, so no renderer or asset server is needed.
    fn fall_while_running_into_wall() -> f32 {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            TransformPlugin,
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(PIXELS_PER_METER),
        ));
        // A fixed step keeps the drop reproducible instead of tying it to how
        // fast the test machine happens to run the app's ticks.
        app.insert_resource(TimestepMode::Fixed {
            dt: STEP,
            substeps: 1,
        });
        app.add_systems(Startup, configure_physics);
        // Stands in for `move_player` holding D down, without an input device.
        app.add_systems(Update, |mut players: Query<&mut Velocity, With<Player>>| {
            for mut velocity in &mut players {
                velocity.linear.x = PLAYER_SPEED;
            }
        });

        // A wall like the ones framing the play area: default friction, which
        // is what the player's own has to be combined against.
        app.world_mut().spawn((
            Transform::from_xyz(WALL_THICKNESS, 0.0, 0.0),
            RigidBody::Fixed,
            Collider::cuboid(WALL_THICKNESS / 2.0, DESIGN_HEIGHT / 2.0),
        ));

        let player = app
            .world_mut()
            .spawn((Player, Transform::default(), physics_body()))
            .id();

        for _ in 0..STEPS {
            // Rapier's fixed step reads the clock, so the world only advances
            // if each tick reports the time that step covers.
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(Duration::from_secs_f32(STEP));
            app.update();
        }

        -app.world()
            .entity(player)
            .get::<Transform>()
            .unwrap()
            .translation
            .y
    }

    #[test]
    fn a_wall_does_not_hold_the_player_up() {
        let fallen = fall_while_running_into_wall();

        // Some loss to the solver is expected; being caught on the wall is not.
        assert!(
            fallen > free_fall() * 0.9,
            "player fell {fallen} against the wall, free fall is {}",
            free_fall()
        );
    }
}
