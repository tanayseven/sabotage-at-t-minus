//! Ladders, and the climbing they exist for.
//!
//! A ladder is the only way between one deck of the rocket and the next, so it
//! has to be forgiving: the player steps into it and holds W, and that is the
//! whole of it. Being on one is decided by where the player is standing rather
//! than by a sensor collider, the same way boarding the rocket is — something
//! you walk straight through is not something the physics pipeline needs to
//! know about, and a collider here would only have to be filtered back out
//! again.
//!
//! While climbing, gravity is switched off and the player drives their own
//! vertical speed. Walking off the side of the ladder is what lets go of it,
//! and the climb stops dead at the deck above rather than being pushed past it,
//! so a player who holds W arrives standing at floor level and stays there.

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::config::{CLIMB_SPEED, PLAYER_HEIGHT};
use crate::player::Player;
use crate::tiles::load_pixel_art;

const LADDER_TILE: &str = "ladder/metal-ladder.png";
/// Behind the player and the crates, so a climb reads as going up the face of
/// the ladder rather than up whatever is in front of it.
const LADDER_Z: f32 = -3.0;

/// How wide a ladder is drawn. Narrower than a door: it is climbed, not walked
/// through.
pub const LADDER_WIDTH: f32 = 48.0;

/// How wide the hole in the deck above has to be, and — the same number, on
/// purpose — how far either side of a ladder it can be held on to.
///
/// The two have to agree. Letting go at the top of a climb is done by walking
/// off the side of the ladder, and that only ever puts the player on solid
/// plate if the reach covers the whole hole. Make the reach the narrower of the
/// two and a player who climbs to the top and steps off drops straight back
/// down the hole they just came up.
pub const LADDER_CLEARANCE: f32 = LADDER_WIDTH + 32.0;

/// Roughly one rung per tile of the art. The run is divided into a whole number
/// of these so the ladder meets both decks flush instead of ending on a
/// half-drawn rung.
const RUNG_SPACING: f32 = 32.0;

/// One run of ladder, from the deck plate it stands on to the one it reaches.
/// Both are given as the *top* of the plate — the surface the player stands on
/// — because that is what a layout is reasoned about, and it is what decides
/// where the climb has to start and stop.
#[derive(Debug, Clone, Copy)]
pub struct Ladder {
    pub x: f32,
    pub foot: f32,
    pub head: f32,
}

/// The patch of air a player counts as being on the ladder in. Lives on its own
/// entity so the climb system can ask about every ladder in the level with one
/// query, and carries [`LevelEntity`](crate::level::LevelEntity) like the rest
/// of the geometry so it is cleared with it.
#[derive(Component)]
pub struct LadderReach(pub Rect);

/// Whether the player has hold of a ladder. Kept on the player rather than
/// worked out twice, so jumping and the animation agree with the climb about
/// what is going on.
#[derive(Component, Default)]
pub struct Climbing(pub bool);

impl Ladder {
    pub const fn new(x: f32, foot: f32, head: f32) -> Self {
        Self { x, foot, head }
    }

    /// Where the ladder can be caught hold of. Its top is half a player above
    /// the deck it serves, which is exactly where the *centre* of a player
    /// standing on that deck sits: the climb runs out at standing height on the
    /// upper floor, with nothing left to push against.
    pub const fn reach(&self) -> Rect {
        Rect {
            min: Vec2::new(self.x - LADDER_CLEARANCE / 2.0, self.foot),
            max: Vec2::new(
                self.x + LADDER_CLEARANCE / 2.0,
                self.head + PLAYER_HEIGHT / 2.0,
            ),
        }
    }

    fn spawn(&self, commands: &mut Commands, rung: &Handle<Image>, marker: impl Bundle + Clone) {
        let run = self.head - self.foot;
        let rungs = (run / RUNG_SPACING).round().max(1.0);
        let spacing = run / rungs;

        for index in 0..rungs as usize {
            let y = self.foot + spacing * (index as f32 + 0.5);

            commands.spawn((
                marker.clone(),
                Sprite {
                    image: rung.clone(),
                    custom_size: Some(Vec2::new(LADDER_WIDTH, spacing)),
                    ..default()
                },
                Transform::from_xyz(self.x, y, LADDER_Z),
            ));
        }

        commands.spawn((marker, LadderReach(self.reach())));
    }
}

pub fn spawn_ladders(
    commands: &mut Commands,
    assets: &AssetServer,
    ladders: &[Ladder],
    marker: impl Bundle + Clone,
) {
    if ladders.is_empty() {
        return;
    }

    let rung = load_pixel_art(assets, LADDER_TILE);

    for ladder in ladders {
        ladder.spawn(commands, &rung, marker.clone());
    }
}

/// Takes hold of a ladder, drives the climb, and lets go again.
///
/// Runs before [`jump`](crate::player::jump) and after
/// [`move_player`](crate::player::move_player): the first because W is both
/// "climb" and "jump" and the ladder has first claim on it, the second because
/// the horizontal speed it writes is what carries the player off the side of a
/// ladder and out of the climb.
pub fn climb_ladder(
    keys: Res<ButtonInput<KeyCode>>,
    ladders: Query<&LadderReach>,
    mut players: Query<(&Transform, &mut Velocity, &mut GravityScale, &mut Climbing), With<Player>>,
) {
    let up = keys.pressed(KeyCode::KeyW);
    let down = keys.pressed(KeyCode::KeyS);

    for (transform, mut velocity, mut gravity, mut climbing) in &mut players {
        let at = transform.translation.truncate();
        let held = ladders
            .iter()
            .map(|reach| reach.0)
            .find(|reach| reach.contains(at));

        // Reaching for the ladder is what starts a climb, so walking past one
        // without touching W or S leaves the player falling normally through it.
        let Some(reach) = held.filter(|_| climbing.0 || up || down) else {
            climbing.0 = false;
            gravity.0 = 1.0;
            continue;
        };

        climbing.0 = true;
        // Held rather than falling: with gravity off, a player touching neither
        // key simply stays where they are on the ladder.
        gravity.0 = 0.0;
        velocity.linear.y = match (up, down) {
            // Stopped at the top rather than pushed past it. Overshooting would
            // drop the player out of the reach, hand them back their weight, and
            // bounce them off the end of the ladder for as long as they held W.
            (true, false) if at.y < reach.max.y => CLIMB_SPEED,
            (false, true) => -CLIMB_SPEED,
            _ => 0.0,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{PIXELS_PER_METER, PLATFORM_HEIGHT};
    use crate::physics::configure_physics;
    use crate::player::{move_player, physics_body};
    use std::time::Duration;

    const STEP: f32 = 1.0 / 60.0;

    /// A ladder between two decks 260 units apart, which is what the rocket's
    /// are. The numbers are the test's own rather than the level's: what is
    /// being checked here is the mechanic, not the layout.
    const FOOT: f32 = 0.0;
    const HEAD: f32 = 260.0;
    const LADDER: Ladder = Ladder::new(0.0, FOOT, HEAD);

    /// Where a player standing on a deck plate has their centre.
    const fn standing_on(deck: f32) -> f32 {
        deck + PLAYER_HEIGHT / 2.0
    }

    #[test]
    fn the_reach_covers_the_hole_the_ladder_comes_through() {
        let reach = LADDER.reach();

        assert_eq!(reach.width(), LADDER_CLEARANCE);
        assert!(reach.width() >= LADDER_WIDTH);
    }

    #[test]
    fn the_reach_runs_from_one_deck_to_standing_height_on_the_next() {
        let reach = LADDER.reach();

        assert!(reach.contains(Vec2::new(LADDER.x, standing_on(FOOT))));
        assert!(reach.contains(Vec2::new(LADDER.x, standing_on(HEAD))));
        assert!(!reach.contains(Vec2::new(LADDER.x, standing_on(HEAD) + 1.0)));
    }

    #[test]
    fn a_player_in_the_next_room_is_not_on_the_ladder() {
        assert!(!LADDER.reach().contains(Vec2::new(300.0, standing_on(FOOT))));
    }

    /// Drives a real climb: two deck plates with a hole in the upper one, a
    /// ladder through it, and a player at the bottom holding W. The whole point
    /// is that the level's own geometry is in the way, so nothing here is
    /// mocked out — Rapier resolves the plates the same as it does in a run.
    fn climb_then_step_off() -> Vec2 {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            TransformPlugin,
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(PIXELS_PER_METER),
        ));
        app.insert_resource(TimestepMode::Fixed {
            dt: STEP,
            substeps: 1,
        });
        app.insert_resource(ButtonInput::<KeyCode>::default());
        app.add_systems(Startup, configure_physics);
        app.add_systems(Update, (move_player, climb_ladder).chain());

        // The deck below, solid, and the deck above with the ladder's hole in it.
        let plate = |centre_x: f32, width: f32, top: f32| {
            (
                Transform::from_xyz(centre_x, top - PLATFORM_HEIGHT / 2.0, 0.0),
                RigidBody::Fixed,
                Collider::cuboid(width / 2.0, PLATFORM_HEIGHT / 2.0),
            )
        };
        let hole = LADDER_CLEARANCE / 2.0;
        app.world_mut().spawn(plate(0.0, 1200.0, FOOT));
        app.world_mut().spawn(plate(-300.0 - hole, 600.0, HEAD));
        app.world_mut().spawn(plate(300.0 + hole, 600.0, HEAD));

        app.world_mut().spawn(LadderReach(LADDER.reach()));

        let player = app
            .world_mut()
            .spawn((
                Player,
                Transform::from_xyz(LADDER.x, standing_on(FOOT), 0.0),
                physics_body(),
            ))
            .id();

        let mut run = |steps: usize, held: &[KeyCode]| {
            let mut keys = ButtonInput::<KeyCode>::default();
            for key in held {
                keys.press(*key);
            }
            app.world_mut().insert_resource(keys);

            for _ in 0..steps {
                app.world_mut()
                    .resource_mut::<Time>()
                    .advance_by(Duration::from_secs_f32(STEP));
                app.update();
            }
        };

        // Long enough to cover the run at CLIMB_SPEED with room to spare, so a
        // climb that stalls part way up shows up as a short finish rather than
        // as the test simply not having waited.
        run(150, &[KeyCode::KeyW]);
        // Then off the ladder, onto the plate, and a moment to settle on it.
        run(60, &[KeyCode::KeyD]);

        app.world()
            .entity(player)
            .get::<Transform>()
            .unwrap()
            .translation
            .truncate()
    }

    #[test]
    fn a_player_climbs_a_ladder_and_steps_off_onto_the_deck_above() {
        let ended = climb_then_step_off();

        // Standing on the upper deck, not fallen back through its hole.
        assert!(
            (ended.y - standing_on(HEAD)).abs() < 6.0,
            "ended at y={}, expected to be standing on the deck at {}",
            ended.y,
            standing_on(HEAD)
        );
        // And clear of the hole rather than balanced over it.
        assert!(
            ended.x > LADDER_CLEARANCE / 2.0,
            "ended at x={}, still over the hole",
            ended.x
        );
    }

    /// Holding W at the top must not push the player off the end of the ladder.
    /// Before the climb was clamped this bounced them in and out of the reach
    /// for as long as the key was down.
    #[test]
    fn the_climb_stops_at_the_deck_it_reaches() {
        let mut app = App::new();
        app.insert_resource(ButtonInput::<KeyCode>::default());
        app.add_systems(Update, climb_ladder);
        app.world_mut().spawn(LadderReach(LADDER.reach()));

        let player = app
            .world_mut()
            .spawn((
                Player,
                Transform::from_xyz(LADDER.x, standing_on(HEAD), 0.0),
                physics_body(),
            ))
            .id();

        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::KeyW);
        app.world_mut().insert_resource(keys);
        app.update();

        let velocity = app.world().entity(player).get::<Velocity>().unwrap();
        assert_eq!(velocity.linear.y, 0.0);
    }
}
