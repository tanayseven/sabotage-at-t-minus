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
///
/// There is a second rule here, and it is the one that bites: **no more than one
/// crate may end up against any one door.** A player crossing a deck pushes what
/// is loose on it along ahead of them and shoves it through the doorway when it
/// opens, so crates gather at the far end of the route rather than staying put.
/// A door has the reach to be worked over one crate; over two it cannot be
/// reached at all, and since a crate will not go through a shut door, the run
/// dead-ends there. That is exactly what a fourth crate in the airlock's own
/// room used to do — the crate pushed in from next door joined it and sealed the
/// way out. The one crate that is not on the route is parked beyond the upper
/// ladder, behind where the player arrives, so it is never pushed anywhere.
/// `a_player_can_cross_the_rocket_from_the_drop_point_to_the_airlock` is what
/// holds this.
const ROCKET_CRATES: [Vec2; 4] = [
    Vec2::new(-300.0, DECK_0 + 140.0),
    Vec2::new(200.0, DECK_1 + 140.0),
    Vec2::new(-180.0, DECK_2 + 140.0),
    Vec2::new(-520.0, DECK_2 + 140.0),
];

/// The far end of the bottom deck's left-hand room — the whole rocket is
/// between the player and the airlock.
const ROCKET_SPAWN: Vec2 = Vec2::new(HULL_LEFT + 120.0, DECK_0 + 60.0);

// ---------------------------------------------------------------------------
// The rooms themselves
// ---------------------------------------------------------------------------

/// Which side of the bulkhead a room is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Port,
    Starboard,
}

impl Side {
    const fn hull(self) -> f32 {
        match self {
            Side::Port => HULL_LEFT,
            Side::Starboard => HULL_RIGHT,
        }
    }

    const fn name(self) -> &'static str {
        match self {
            Side::Port => "port",
            Side::Starboard => "starboard",
        }
    }
}

/// How many decks the rocket has, and how many rooms the bulkhead cuts each of
/// them into.
const DECK_COUNT: usize = 3;
const ROOMS_PER_DECK: usize = 2;
/// Every room in the rocket, which is what anything picking one at random works
/// against.
pub const ROOM_COUNT: usize = DECK_COUNT * ROOMS_PER_DECK;

/// One of the rocket's six rooms: the stretch of a deck on one side of the
/// bulkhead. Described by which deck and which side rather than by its corners,
/// because that is what a room *is* here — the plates, the hull and the
/// bulkhead already say where the walls are, and a second copy of those numbers
/// would only be one to keep in step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Room {
    /// 0 is the deck the player is dropped onto, 2 the one the airlock is on.
    pub deck: usize,
    pub side: Side,
}

/// How far along a room, from the bulkhead toward the hull, a fixture is
/// mounted. Deliberately not the middle: a ladder comes up at 400 units out and
/// a doorway is worked from as far as a crate's width back from the bulkhead,
/// so the middle of the room is the one place a fixture would be in the way of
/// both. `a_panel_is_clear_of_everything_else_in_its_room` holds this.
const FIXTURE_ALONG_ROOM: f32 = 0.45;

impl Room {
    /// The `index`th room, counting up the rocket a deck at a time. What makes
    /// a room pickable with a single number.
    pub const fn from_index(index: usize) -> Self {
        Self {
            deck: index / ROOMS_PER_DECK,
            side: if index.is_multiple_of(ROOMS_PER_DECK) {
                Side::Port
            } else {
                Side::Starboard
            },
        }
    }

    /// The deck plate the room is floored with — the surface walked on.
    pub const fn floor(self) -> f32 {
        DECK_0 + self.deck as f32 * DECK_HEIGHT
    }

    /// Where a wall fixture is mounted in this room, given as the point on the
    /// floor it stands on, so what is hung there decides its own height.
    pub const fn fixture(self) -> Vec2 {
        Vec2::new(
            BULKHEAD_X + (self.side.hull() - BULKHEAD_X) * FIXTURE_ALONG_ROOM,
            self.floor(),
        )
    }

    /// How the room is named in anything the player reads.
    pub fn label(self) -> String {
        format!("deck {}, {}", self.deck, self.side.name())
    }
}

const ROCKET_ROOMS: [Room; ROOM_COUNT] = [
    Room::from_index(0),
    Room::from_index(1),
    Room::from_index(2),
    Room::from_index(3),
    Room::from_index(4),
    Room::from_index(5),
];

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

    /// The rooms the level is divided into. Only the rocket has any: the ascent
    /// is open ground, with nothing to be inside of.
    pub fn rooms(self) -> &'static [Room] {
        match self {
            Level::Rocket => &ROCKET_ROOMS,
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

    /// Drives a whole run through the rocket with scripted input, against the
    /// real level: [`build_level`] puts up the same walls, plates, ladders,
    /// doors and crates a player gets, and the same systems move the character
    /// through them. What it is really for is the joins — a door that opens onto
    /// a ladder that comes up in a room whose door is out of reach is a run that
    /// dead-ends, and every piece of that passes its own test on its own.
    mod crossing {
        use super::*;
        use crate::config::PIXELS_PER_METER;
        use crate::door::{leave_through_airlock, use_doors};
        use crate::ladder::climb_ladder;
        use crate::panel::Panel;
        use crate::physics::configure_physics;
        use crate::player::{Player, jump, move_player, probe_ground};
        use crate::setup::build_level;
        use bevy::asset::AssetPlugin;
        use bevy_rapier2d::prelude::*;
        use std::time::Duration;

        const STEP: f32 = 1.0 / 60.0;
        const A: KeyCode = KeyCode::KeyA;
        const D: KeyCode = KeyCode::KeyD;
        const E: KeyCode = KeyCode::KeyE;
        const W: KeyCode = KeyCode::KeyW;

        /// Where a player standing on a deck has their centre.
        fn standing_on(deck: f32) -> f32 {
            deck + PLAYER_HEIGHT / 2.0
        }

        struct Run {
            app: App,
            player: Entity,
        }

        impl Run {
            fn start() -> Self {
                let mut app = App::new();
                app.add_plugins((
                    MinimalPlugins,
                    AssetPlugin::default(),
                    TransformPlugin,
                    RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(PIXELS_PER_METER),
                ));
                app.insert_resource(TimestepMode::Fixed {
                    dt: STEP,
                    substeps: 1,
                });
                app.insert_resource(ButtonInput::<KeyCode>::default());
                // The art is never rendered here, but the level still asks the
                // asset server for it, and a handle cannot be handed out for a
                // type the app has never heard of.
                app.init_asset::<Image>();
                app.init_resource::<Level>();
                app.add_systems(Startup, configure_physics);
                // The panel is built along with the rest of it, and pinned to a
                // seed so the crossing is run against a rocket with one in it
                // rather than one where the pick happened to go elsewhere.
                app.init_resource::<Panel>();
                app.add_systems(
                    Startup,
                    |mut commands: Commands, assets: Res<AssetServer>, panel: Res<Panel>| {
                        build_level(&mut commands, &assets, Level::Rocket, &panel);
                    },
                );
                app.add_systems(
                    Update,
                    (
                        move_player,
                        probe_ground,
                        climb_ladder,
                        jump,
                        use_doors,
                        leave_through_airlock,
                    )
                        .chain(),
                );
                app.update();

                let player = {
                    let mut query = app.world_mut().query_filtered::<Entity, With<Player>>();
                    query.iter(app.world()).next().expect("no player was built")
                };

                Self { app, player }
            }

            fn at(&self) -> Vec2 {
                self.app
                    .world()
                    .entity(self.player)
                    .get::<Transform>()
                    .expect("the player lost its transform")
                    .translation
                    .truncate()
            }

            fn step(&mut self, held: &[KeyCode]) {
                let mut keys = ButtonInput::<KeyCode>::default();
                for key in held {
                    keys.press(*key);
                }
                self.app.world_mut().insert_resource(keys);
                self.app
                    .world_mut()
                    .resource_mut::<Time>()
                    .advance_by(Duration::from_secs_f32(STEP));
                self.app.update();
            }

            fn hold(&mut self, held: &[KeyCode], steps: usize) {
                for _ in 0..steps {
                    self.step(held);
                }
            }

            /// Holds the keys until the player is where `arrived` wants them, or
            /// gives up. Reported rather than asserted so the caller can say what
            /// it was that never happened.
            fn hold_until(
                &mut self,
                held: &[KeyCode],
                budget: usize,
                arrived: impl Fn(Vec2) -> bool,
            ) -> bool {
                for _ in 0..budget {
                    self.step(held);
                    if arrived(self.at()) {
                        return true;
                    }
                }
                false
            }

            /// Walks into a shut door and reports where it brought the player up.
            fn shut_out_by_the_door(&mut self, key: KeyCode) -> Vec2 {
                self.hold(&[key], 400);
                self.at()
            }

            fn level(&self) -> Level {
                *self.app.world().resource::<Level>()
            }
        }

        /// A shut door has to do two things, and the second is the one that was
        /// wrong: stop the player, and stop them somewhere they can still work it
        /// from. How far back that is depends on what they pushed there ahead of
        /// them, so it is asked of the door itself rather than measured against a
        /// distance picked by hand.
        fn assert_shut_out(at: Vec2, approaching: KeyCode, deck: f32, name: &str) {
            let door = Door::bulkhead(BULKHEAD_X, deck);
            let short_of_it = match approaching {
                D => at.x < BULKHEAD_X,
                _ => at.x > BULKHEAD_X,
            };

            assert!(short_of_it, "{name}: walked through a shut door to {at:?}");
            assert!(
                door.in_reach(at),
                "{name}: brought up at {at:?}, too far off the door to work it"
            );
        }

        #[test]
        fn a_player_can_cross_the_rocket_from_the_drop_point_to_the_airlock() {
            let mut run = Run::start();

            // The bottom deck: walk out of the drop point into the bulkhead.
            let at = run.shut_out_by_the_door(D);
            assert_shut_out(at, D, DECK_0, "deck 0");

            // Work it, carry on, and take the ladder up.
            run.hold(&[E], 2);
            assert!(
                run.hold_until(&[D], 600, |at| at.x >= LOWER_LADDER_X),
                "deck 0's door never let the player through to the lower ladder (stuck at {:?})",
                run.at()
            );
            assert!(
                run.hold_until(&[W], 400, |at| at.y >= standing_on(DECK_1) - 1.0),
                "the lower ladder never reached deck 1"
            );

            // Off the ladder onto solid plate, then back across to deck 1's door.
            assert!(
                run.hold_until(&[A], 300, |at| at.x <= LOWER_LADDER_X - LADDER_GAP),
                "never stepped off the lower ladder onto deck 1"
            );
            let at = run.shut_out_by_the_door(A);
            assert_shut_out(at, A, DECK_1, "deck 1");

            // Through it and up the second ladder.
            run.hold(&[E], 2);
            assert!(
                run.hold_until(&[A], 600, |at| at.x <= UPPER_LADDER_X),
                "deck 1's door never let the player through to the upper ladder"
            );
            assert!(
                run.hold_until(&[W], 400, |at| at.y >= standing_on(DECK_2) - 1.0),
                "the upper ladder never reached deck 2"
            );

            // Off it, across the top deck, and through the last bulkhead.
            assert!(
                run.hold_until(&[D], 300, |at| at.x >= UPPER_LADDER_X + LADDER_GAP),
                "never stepped off the upper ladder onto deck 2"
            );
            let at = run.shut_out_by_the_door(D);
            assert_shut_out(at, D, DECK_2, "deck 2");

            run.hold(&[E], 2);
            assert!(
                run.hold_until(&[D], 600, |at| AIRLOCK.in_reach(at)),
                "deck 2's door never let the player within reach of the airlock (stuck at {:?})",
                run.at()
            );

            assert_eq!(
                run.level(),
                Level::Rocket,
                "the run left the rocket before the airlock was worked"
            );

            // And out. Working the airlock while standing in it is what ends the
            // level, so the walk and the press go in together.
            run.hold(&[E, D], 6);

            assert_eq!(
                run.level(),
                Level::Ascent,
                "the airlock did not put the run out onto the ascent"
            );
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
