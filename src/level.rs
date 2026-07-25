//! The levels a run is made of, and what makes each one different.
//!
//! A run starts inside the rocket: three decks of rooms stacked up the inside
//! of the hull, split left and right by bulkheads with doors in them and joined
//! floor to floor by ladders. It is walled in on every side, and the camera
//! zooms in and pans around it rather than holding the whole thing in frame.
//!
//! Past it lies the ascent: open ground with no walls at all. Stepping into the
//! rocket's airlock swaps the geometry for it without ending the run — the HUD
//! and the mission clock carry straight over.

use bevy::prelude::*;

use crate::config::{DESIGN_HEIGHT, FOLLOW_ZOOM, INTERIOR_ZOOM, PLATFORM_HEIGHT, WALL_THICKNESS};
use crate::door::Door;
use crate::ladder::{LADDER_CLEARANCE, Ladder};
use crate::platform::Platform;
use crate::wall::Wall;

/// Marks the level geometry: walls, platforms, ladders, doors, crates and the
/// player. Cleared both when the run ends and when it moves on to the next
/// level, which is what separates it from the HUD's
/// [`crate::setup::GameEntity`].
#[derive(Component, Clone)]
pub struct LevelEntity;

/// Which scene the current run is in. Levels run in the order below, and the
/// mission clock spans the whole run rather than restarting on each one.
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    /// Where the run starts: the rooms inside the rocket, boxed in by the hull.
    #[default]
    Rocket,
    /// Open ground outside it. No walls, and the camera zooms in and tracks the
    /// player instead of keeping the level still.
    Ascent,
}

/// How the camera frames a level.
#[derive(Debug, Clone, Copy)]
pub enum CameraMode {
    /// The level is built to the size of the viewport, so the camera holds
    /// still at the origin and shows all of it.
    Fixed,
    /// Zoomed in by `zoom` and following the player, never panning past
    /// `bounds`.
    Follow { zoom: f32, bounds: Rect },
}

impl CameraMode {
    pub fn zoom(self) -> f32 {
        match self {
            CameraMode::Fixed => 1.0,
            CameraMode::Follow { zoom, .. } => zoom,
        }
    }
}

/// Top of the floor, shared by the rocket's bottom deck and the ascent's ground.
const GROUND_TOP: f32 = -DESIGN_HEIGHT / 2.0;

// ---------------------------------------------------------------------------
// The rocket's rooms
// ---------------------------------------------------------------------------

/// Inside faces of the hull. Everything in the rocket is laid out between them.
const HULL_LEFT: f32 = -600.0;
const HULL_RIGHT: f32 = 600.0;

/// The bulkhead that splits every deck into two rooms. Down the middle, so both
/// rooms on a deck are the same size and neither is a corridor.
const BULKHEAD_X: f32 = 0.0;

/// Floor to floor. The [`PLATFORM_HEIGHT`] of it that is deck plate leaves 228
/// units of headroom, which swallows a door and still clears a standing player
/// by a wide margin — `a_deck_has_the_headroom_for_its_doors` holds it to that.
const DECK_HEIGHT: f32 = 260.0;

/// The three deck plates, by the surface the player walks on. Deck 0 is the one
/// they are dropped onto and deck 2 is the one the airlock is on.
const DECK_0: f32 = GROUND_TOP;
const DECK_1: f32 = DECK_0 + DECK_HEIGHT;
const DECK_2: f32 = DECK_1 + DECK_HEIGHT;
/// Underside of the nose cap, closing the top deck in.
const ROCKET_CEILING: f32 = DECK_2 + DECK_HEIGHT;

/// The ladders are put on opposite sides of the rocket on purpose: coming up
/// one leaves the player at the far end of the deck from the next, so every
/// room on the way up is walked through rather than passed by.
const LOWER_LADDER_X: f32 = 400.0;
const UPPER_LADDER_X: f32 = -400.0;

/// The hole in the deck plate a ladder comes up through. It is exactly the
/// ladder's reach, which is what puts a player who steps off at the top on
/// solid plate — see [`LADDER_CLEARANCE`].
const LADDER_GAP: f32 = LADDER_CLEARANCE;

/// The way out, set into the hull on the top deck.
const AIRLOCK_X: f32 = HULL_RIGHT - WALL_THICKNESS / 2.0;

/// A stretch of deck plate spanning `from` to `to`. Decks are described by the
/// two ends of each run of plate, because what a deck actually is here is a
/// floor with a hole in it, and the holes are what matter.
const fn plate(from: f32, to: f32, top: f32) -> Platform {
    Platform::with_top((from + to) / 2.0, top, to - from)
}

const ROCKET_PLATFORMS: [Platform; 6] = [
    // The bottom deck is solid: nothing goes below it.
    plate(HULL_LEFT, HULL_RIGHT, DECK_0),
    // Deck 1, opened up where the lower ladder comes through.
    plate(HULL_LEFT, LOWER_LADDER_X - LADDER_GAP / 2.0, DECK_1),
    plate(LOWER_LADDER_X + LADDER_GAP / 2.0, HULL_RIGHT, DECK_1),
    // Deck 2, opened up where the upper one does.
    plate(HULL_LEFT, UPPER_LADDER_X - LADDER_GAP / 2.0, DECK_2),
    plate(UPPER_LADDER_X + LADDER_GAP / 2.0, HULL_RIGHT, DECK_2),
    // The nose cap. Without it a good jump on the top deck leaves the rocket.
    plate(HULL_LEFT, HULL_RIGHT, ROCKET_CEILING),
];

/// One door per deck through the bulkhead, and the way out on the top one.
const DECK_0_DOOR: Door = Door::bulkhead(BULKHEAD_X, DECK_0);
const DECK_1_DOOR: Door = Door::bulkhead(BULKHEAD_X, DECK_1);
const DECK_2_DOOR: Door = Door::bulkhead(BULKHEAD_X, DECK_2);
const AIRLOCK: Door = Door::airlock(AIRLOCK_X, DECK_2);

const ROCKET_DOORS: [Door; 4] = [DECK_0_DOOR, DECK_1_DOOR, DECK_2_DOOR, AIRLOCK];

const ROCKET_WALLS: [Wall; 5] = [
    Wall::between(HULL_LEFT, DECK_0, ROCKET_CEILING),
    Wall::between(HULL_RIGHT, DECK_0, ROCKET_CEILING),
    // Each bulkhead picks up where its deck's doorway leaves off and runs to the
    // underside of the deck above, so the door is the only way through it.
    Wall::between(BULKHEAD_X, DECK_0_DOOR.lintel(), DECK_1 - PLATFORM_HEIGHT),
    Wall::between(BULKHEAD_X, DECK_1_DOOR.lintel(), DECK_2 - PLATFORM_HEIGHT),
    Wall::between(
        BULKHEAD_X,
        DECK_2_DOOR.lintel(),
        ROCKET_CEILING - PLATFORM_HEIGHT,
    ),
];

const ROCKET_LADDERS: [Ladder; 2] = [
    Ladder::new(LOWER_LADDER_X, DECK_0, DECK_1),
    Ladder::new(UPPER_LADDER_X, DECK_1, DECK_2),
];

/// Kept clear of the doorways and the ladder holes, so nothing settles where it
/// would wedge the player's way through.
const ROCKET_CRATES: [Vec2; 4] = [
    Vec2::new(-300.0, DECK_0 + 140.0),
    Vec2::new(200.0, DECK_1 + 140.0),
    Vec2::new(-180.0, DECK_2 + 140.0),
    Vec2::new(320.0, DECK_2 + 140.0),
];

/// The far end of the bottom deck's left-hand room — the whole rocket is
/// between the player and the airlock.
const ROCKET_SPAWN: Vec2 = Vec2::new(HULL_LEFT + 120.0, DECK_0 + 60.0);

// ---------------------------------------------------------------------------
// The ascent
// ---------------------------------------------------------------------------

/// How far the ascent reaches either side of the origin — the line the camera
/// stops panning at, not a wall. Nothing stops the player walking past it.
const ASCENT_REACH: f32 = 2100.0;
/// Headroom above the ground for the camera to climb into, with room to spare
/// over the highest ledge so the top of the climb is not framed against nothing.
const ASCENT_CEILING: f32 = GROUND_TOP + 1400.0;

/// The ascent has to be laid out for a camera that only ever shows a slice of
/// it: at [`FOLLOW_ZOOM`] roughly 710x400 units are on screen at once. Two rules
/// fall out of that, and both were learned the hard way from a layout drawn for
/// the unzoomed view.
///
/// The first is that the ledges have to be close together — near enough that
/// two or three are always in frame, or the player runs through blank screen.
///
/// The second is that height has to be spent carefully. Standing on the ground
/// the camera is clamped to the bottom of the level, so anything more than
/// ~370 units up is off the top of the screen. The long run therefore stays in
/// a shallow band where every ledge is visible from the floor, and the climb
/// that does go up is saved for the end, where going up *is* the point.
/// The third rule, and the one that decides these numbers: a jump at a full run
/// covers about 400 units, so a ledge much narrower than that is one a player
/// holding "right" sails straight over. The ledges are wide and the gaps between
/// them small — the run is meant to be a rhythm, not a series of pixel landings.
const ASCENT_BAND_STEP: f32 = 360.0;
const ASCENT_BAND_WIDTH: f32 = 330.0;
const ASCENT_BAND_START: f32 = -1600.0;

/// One ledge of the long run, `index` steps along, `height` units above the
/// ground. Heights roll up and down rather than climbing, so the camera drifts
/// vertically as the player hops along instead of holding one line.
const fn band(index: f32, height: f32) -> Platform {
    Platform::with_top(
        ASCENT_BAND_START + index * ASCENT_BAND_STEP,
        GROUND_TOP + height,
        ASCENT_BAND_WIDTH,
    )
}

/// Where the climb at the end of the run starts, past the last band ledge.
const ASCENT_CLIMB_START: f32 = 1300.0;
const ASCENT_CLIMB_STEP: Vec2 = Vec2::new(220.0, 130.0);
const ASCENT_CLIMB_WIDTH: f32 = 220.0;

/// One rung of that climb. Tighter and steeper than the band: 20 units of gap
/// and 130 of rise, against a jump arc of 225 up by ~400 across.
const fn rung(index: f32) -> Platform {
    Platform::with_top(
        ASCENT_CLIMB_START + index * ASCENT_CLIMB_STEP.x,
        GROUND_TOP + 480.0 + index * ASCENT_CLIMB_STEP.y,
        ASCENT_CLIMB_WIDTH,
    )
}

const ASCENT_PLATFORMS: [Platform; 13] = [
    // With no side walls, the ground is the only thing between the player and
    // open air, so it runs well past the far edge of the camera's reach.
    Platform::with_top(0.0, GROUND_TOP, ASCENT_REACH * 2.0 + 400.0),
    // The first ledge clears a player standing on the ground by a good margin.
    // Lower, and running along the floor snags on its underside.
    band(0.0, 180.0),
    band(1.0, 280.0),
    band(2.0, 210.0),
    band(3.0, 320.0),
    band(4.0, 240.0),
    band(5.0, 350.0),
    band(6.0, 260.0),
    band(7.0, 360.0),
    rung(0.0),
    rung(1.0),
    rung(2.0),
    rung(3.0),
];

const ASCENT_CRATES: [Vec2; 5] = [
    Vec2::new(band(1.0, 280.0).centre.x, band(1.0, 280.0).top() + 120.0),
    Vec2::new(band(3.0, 320.0).centre.x, band(3.0, 320.0).top() + 120.0),
    Vec2::new(band(5.0, 350.0).centre.x, band(5.0, 350.0).top() + 120.0),
    Vec2::new(band(7.0, 360.0).centre.x, band(7.0, 360.0).top() + 120.0),
    Vec2::new(rung(2.0).centre.x, rung(2.0).top() + 120.0),
];

impl Level {
    /// The level that follows this one, or `None` at the end of the run.
    pub fn next(self) -> Option<Self> {
        match self {
            Level::Rocket => Some(Level::Ascent),
            Level::Ascent => None,
        }
    }

    pub fn platforms(self) -> &'static [Platform] {
        match self {
            Level::Rocket => &ROCKET_PLATFORMS,
            Level::Ascent => &ASCENT_PLATFORMS,
        }
    }

    /// The hull and the bulkheads. The ascent is deliberately open ground, which
    /// is what makes it feel like the outside.
    pub fn walls(self) -> &'static [Wall] {
        match self {
            Level::Rocket => &ROCKET_WALLS,
            Level::Ascent => &[],
        }
    }

    pub fn ladders(self) -> &'static [Ladder] {
        match self {
            Level::Rocket => &ROCKET_LADDERS,
            Level::Ascent => &[],
        }
    }

    pub fn doors(self) -> &'static [Door] {
        match self {
            Level::Rocket => &ROCKET_DOORS,
            Level::Ascent => &[],
        }
    }

    pub fn crates(self) -> &'static [Vec2] {
        match self {
            Level::Rocket => &ROCKET_CRATES,
            Level::Ascent => &ASCENT_CRATES,
        }
    }

    pub fn player_spawn(self) -> Vec2 {
        match self {
            Level::Rocket => ROCKET_SPAWN,
            Level::Ascent => Vec2::new(ASCENT_BAND_START - 450.0, GROUND_TOP + 60.0),
        }
    }

    pub fn camera(self) -> CameraMode {
        match self {
            // Zoomed in enough that a room fills the frame, which is what makes
            // the inside of the rocket read as rooms rather than as a diagram of
            // one. Both axes pan: the hull is wider and taller than the viewport.
            Level::Rocket => CameraMode::Follow {
                zoom: INTERIOR_ZOOM,
                bounds: Rect::new(
                    HULL_LEFT - WALL_THICKNESS,
                    DECK_0 - PLATFORM_HEIGHT,
                    HULL_RIGHT + WALL_THICKNESS,
                    ROCKET_CEILING,
                ),
            },
            Level::Ascent => CameraMode::Follow {
                zoom: FOLLOW_ZOOM,
                bounds: Rect::new(
                    -ASCENT_REACH,
                    GROUND_TOP - PLATFORM_HEIGHT,
                    ASCENT_REACH,
                    ASCENT_CEILING,
                ),
            },
        }
    }
}

/// Inserted rather than assigned, so change detection fires — and the camera
/// re-frames — even when a new run starts on the level the last one ended on.
pub fn reset_level(mut commands: Commands) {
    commands.insert_resource(Level::default());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{PLAYER_HEIGHT, PLAYER_WIDTH};
    use crate::door::DOOR_SIZE;
    use crate::ladder::LADDER_WIDTH;

    /// The horizontal span a stretch of deck plate covers.
    fn span(platform: &Platform) -> (f32, f32) {
        (
            platform.centre.x - platform.width / 2.0,
            platform.centre.x + platform.width / 2.0,
        )
    }

    #[test]
    fn a_run_opens_inside_the_rocket_and_leaves_by_the_ascent() {
        assert_eq!(Level::default(), Level::Rocket);
        assert_eq!(Level::Rocket.next(), Some(Level::Ascent));
        assert_eq!(Level::Ascent.next(), None);
    }

    #[test]
    fn only_the_rocket_is_walled_in() {
        assert!(!Level::Rocket.walls().is_empty());
        assert!(Level::Ascent.walls().is_empty());
        assert!(Level::Ascent.ladders().is_empty());
        assert!(Level::Ascent.doors().is_empty());
    }

    /// The failure that makes a ladder useless: a deck plate laid across the top
    /// of it, so the climb ends against a ceiling and the room above is sealed.
    #[test]
    fn every_ladder_comes_up_through_a_hole_in_the_deck() {
        for ladder in &ROCKET_LADDERS {
            let column = ladder.reach();

            for plate in &ROCKET_PLATFORMS {
                // Only the plates the ladder has to pass through count — not the
                // one it stands on, and not the ones further up the rocket.
                let in_the_way = plate.top() > ladder.foot && plate.top() <= ladder.head;
                if !in_the_way {
                    continue;
                }

                let (left, right) = span(plate);
                assert!(
                    right <= column.min.x || left >= column.max.x,
                    "{plate:?} is laid across the ladder at x={}",
                    ladder.x
                );
            }
        }
    }

    /// The other half of it: a ladder with no floor under its foot is one the
    /// player can never reach to start climbing.
    #[test]
    fn every_ladder_stands_on_solid_deck_plate() {
        for ladder in &ROCKET_LADDERS {
            let stood_on = ROCKET_PLATFORMS.iter().any(|plate| {
                let (left, right) = span(plate);

                plate.top() == ladder.foot && left < ladder.x && right > ladder.x
            });

            assert!(stood_on, "the ladder at x={} stands on nothing", ladder.x);
        }
    }

    /// Every door has to sit on a deck the player can actually stand on, or its
    /// sill is somewhere in mid-air.
    #[test]
    fn every_door_stands_on_a_deck() {
        for door in &ROCKET_DOORS {
            assert!(
                [DECK_0, DECK_1, DECK_2].contains(&door.sill()),
                "{door:?} does not stand on a deck"
            );
        }
    }

    /// A deck has to swallow a whole doorway and still leave a bulkhead above
    /// it. Raise the doors or lower the decks far enough and the two meet, which
    /// would leave a bulkhead of negative length holding nothing up.
    #[test]
    fn a_deck_has_the_headroom_for_its_doors() {
        let headroom = DECK_HEIGHT - PLATFORM_HEIGHT;

        assert!(
            headroom > DOOR_SIZE.y,
            "a doorway does not fit under a deck"
        );
        const {
            assert!(
                DOOR_SIZE.y > PLAYER_HEIGHT,
                "a doorway is too low to walk through"
            )
        };
    }

    /// The rocket is a box. A gap anywhere in the hull is a way out of the level
    /// that is not the airlock.
    #[test]
    fn the_hull_closes_the_rocket_in() {
        let floor = &ROCKET_PLATFORMS[0];
        let cap = &ROCKET_PLATFORMS[ROCKET_PLATFORMS.len() - 1];

        assert_eq!(floor.top(), DECK_0);
        assert_eq!(cap.top(), ROCKET_CEILING);
        assert_eq!(span(floor), (HULL_LEFT, HULL_RIGHT));
        assert_eq!(span(cap), (HULL_LEFT, HULL_RIGHT));

        for side in [HULL_LEFT, HULL_RIGHT] {
            let closed = ROCKET_WALLS.iter().any(|wall| {
                wall.centre.x == side
                    && wall.centre.y - wall.length / 2.0 <= DECK_0
                    && wall.centre.y + wall.length / 2.0 >= ROCKET_CEILING
            });

            assert!(closed, "the hull is open at x={side}");
        }
    }

    /// The rooms have to be wide enough to be rooms. A bulkhead door and a
    /// ladder in the same room want a player's width between them at least.
    #[test]
    fn the_ladders_are_clear_of_the_bulkhead_and_the_hull() {
        for ladder in &ROCKET_LADDERS {
            let clearance = PLAYER_WIDTH + LADDER_WIDTH / 2.0;

            assert!(
                (ladder.x - BULKHEAD_X).abs() > clearance + DOOR_SIZE.x / 2.0,
                "the ladder at x={} is on top of the bulkhead door",
                ladder.x
            );
            assert!(
                ladder.x - HULL_LEFT > clearance && HULL_RIGHT - ladder.x > clearance,
                "the ladder at x={} is jammed against the hull",
                ladder.x
            );
        }
    }

    #[test]
    fn the_player_spawns_on_the_bottom_deck_away_from_the_airlock() {
        let spawn = Level::Rocket.player_spawn();

        assert!(spawn.y > DECK_0 && spawn.y < DECK_1);
        // On the far side of the bulkhead from the way out, so the run crosses
        // every room rather than starting next to the exit.
        assert!(spawn.x < BULKHEAD_X);
        const { assert!(AIRLOCK_X > BULKHEAD_X, "the airlock is on the spawn's side") };
    }

    /// A spawn outside the camera bounds would start the level with the player
    /// off screen, since the camera never pans past them.
    #[test]
    fn every_level_spawns_the_player_inside_its_camera_bounds() {
        for level in [Level::Rocket, Level::Ascent] {
            let CameraMode::Follow { bounds, .. } = level.camera() else {
                panic!("{level:?} is meant to use a following camera");
            };

            assert!(bounds.contains(level.player_spawn()));
        }
    }

    #[test]
    fn the_ascent_tracks_the_player() {
        assert!(matches!(
            Level::Ascent.camera(),
            CameraMode::Follow { zoom, .. } if zoom > 1.0
        ));
    }

    /// Every ledge of the ascent has to be reachable from the one before it, or
    /// the run dead-ends part way along.
    #[test]
    fn the_ascent_can_be_climbed() {
        use crate::config::{GRAVITY, JUMP_SPEED, PLAYER_SPEED};

        let rise = JUMP_SPEED * JUMP_SPEED / (2.0 * GRAVITY);
        let reach = PLAYER_SPEED * 2.0 * JUMP_SPEED / GRAVITY;

        // Skipping the ground, which every ledge sits above rather than after.
        for pair in ASCENT_PLATFORMS[1..].windows(2) {
            let (from, to) = (&pair[0], &pair[1]);
            let gap = (to.centre.x - to.width / 2.0) - (from.centre.x + from.width / 2.0);

            assert!(
                to.top() - from.top() <= rise,
                "unreachable rise onto {to:?}"
            );
            assert!(gap <= reach, "unjumpable gap before {to:?}");
        }
    }

    /// The lesson the first draft of this level taught: at [`FOLLOW_ZOOM`] the
    /// camera sits on the floor of the level while the player is on the ground,
    /// so a ledge much above it is simply not on screen. The long run has to
    /// stay under that line or the player runs through an empty frame.
    #[test]
    fn the_long_run_is_visible_from_the_ground() {
        use crate::config::VIEW_HEIGHT;

        let CameraMode::Follow { bounds, .. } = Level::Ascent.camera() else {
            panic!("the ascent is meant to use a following camera");
        };
        // The narrowest the viewport ever gets vertically, and so the least
        // that is ever on screen.
        let half_view = VIEW_HEIGHT / FOLLOW_ZOOM / 2.0;
        let camera_y = (GROUND_TOP + PLAYER_HEIGHT / 2.0)
            .clamp(bounds.min.y + half_view, bounds.max.y - half_view);
        let top_of_screen = camera_y + half_view;

        let long_run = &ASCENT_PLATFORMS[1..=8];
        for ledge in long_run {
            assert!(
                ledge.top() <= top_of_screen,
                "{ledge:?} is off the top of the screen"
            );
        }
    }

    /// What wedged the player against the first ledge on the first play-through:
    /// a ledge slung low enough that its underside catches someone running along
    /// the floor. Every ledge has to clear a standing player.
    #[test]
    fn nothing_in_the_ascent_hangs_low_enough_to_snag_on() {
        for ledge in &ASCENT_PLATFORMS[1..] {
            let underside = ledge.top() - PLATFORM_HEIGHT;
            let head = GROUND_TOP + PLAYER_HEIGHT;

            assert!(underside >= head, "{ledge:?} hangs into head height");
        }
    }

    /// The ascent has no side walls, so its ground has to reach past both edges
    /// of the camera's travel or the player can walk off the end of the world.
    #[test]
    fn the_ascent_ground_runs_past_the_camera_bounds() {
        let CameraMode::Follow { bounds, .. } = Level::Ascent.camera() else {
            panic!("the ascent is meant to use a following camera");
        };
        let ground = &ASCENT_PLATFORMS[0];
        let half_width = ground.width / 2.0;

        assert!(ground.centre.x - half_width < bounds.min.x);
        assert!(ground.centre.x + half_width > bounds.max.x);
    }
}
