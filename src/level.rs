//! The levels a run is made of, and what makes each one different.
//!
//! A run starts inside the rocket. Leaving through the airlock transitions to
//! the ascent, and then to the upper deck through the portal/minigame flow.

use bevy::prelude::*;

use crate::config::{DESIGN_HEIGHT, FOLLOW_ZOOM, INTERIOR_ZOOM, PLATFORM_HEIGHT, WALL_THICKNESS};
use crate::door::Door;
use crate::ladder::{LADDER_CLEARANCE, Ladder};
use crate::minigames::{CompletedMinigame, MINIGAME_COUNT, MinigameId, MinigameOutcome};
use crate::platform::Platform;
use crate::portal::TriggeredPortal;
use crate::state::PlayingState;
use crate::wall::Wall;

/// Marks the level geometry: walls, platforms, ladders, doors, crates and the
/// player. Cleared both when the run ends and when it moves on to the next
/// level, which is what separates it from the HUD's
/// [`crate::setup::GameEntity`].
#[derive(Component, Clone)]
pub struct LevelEntity;

/// Which scene the current run is in. Levels run in the order below, and the
/// mission clock resets for each level.
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    /// Where the run starts: rooms inside the rocket.
    #[default]
    Rocket,
    /// Open ground outside the rocket.
    Ascent,
    /// The third stage reached after clearing the ascent's portal challenge.
    UpperDeck,
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

#[derive(Clone, Copy)]
pub struct LevelConfig {
    pub platforms: &'static [Platform],
    pub walls: &'static [Wall],
    pub ladders: &'static [Ladder],
    pub doors: &'static [Door],
    pub crates: &'static [Vec2],
    pub player_spawn: Vec2,
    pub camera: CameraMode,
    pub portal_positions: &'static [Vec2],
    pub portal_minigames: &'static [MinigameId],
}

/// How much of the current level's portal objective has been completed.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LevelProgress {
    pub total_portals: usize,
    pub completed_portals: usize,
}

impl LevelProgress {
    pub fn new(level: Level) -> Self {
        Self {
            total_portals: level.portal_count(),
            completed_portals: 0,
        }
    }

    pub fn complete_portal(&mut self) -> bool {
        self.completed_portals = self.completed_portals.saturating_add(1);
        self.all_portals_completed()
    }

    pub fn all_portals_completed(&self) -> bool {
        self.completed_portals >= self.total_portals
    }

    /// A level is solved when every obstacle type present on that level has
    /// been solved: the panel challenge (if this level has it) and all portal
    /// challenges (if any portals are present).
    pub fn all_obstacles_completed(
        &self,
        level: Level,
        panel_room: Room,
        panel_solved: bool,
    ) -> bool {
        let panel_done = if level.rooms().contains(&panel_room) {
            panel_solved
        } else {
            true
        };

        panel_done && self.all_portals_completed()
    }
}

/// Top of the floor, shared by the rocket's bottom deck and the outside ground.
const GROUND_TOP: f32 = -DESIGN_HEIGHT / 2.0;
const FOLLOW_BOUNDS: Rect = Rect::new(
    -2100.0,
    GROUND_TOP - PLATFORM_HEIGHT,
    2100.0,
    GROUND_TOP + 1400.0,
);

// ---------------------------------------------------------------------------
// Rocket level
// ---------------------------------------------------------------------------

const HULL_LEFT: f32 = -600.0;
const HULL_RIGHT: f32 = 600.0;
const BULKHEAD_X: f32 = 0.0;

const DECK_HEIGHT: f32 = 260.0;
const DECK_0: f32 = GROUND_TOP;
const DECK_1: f32 = DECK_0 + DECK_HEIGHT;
const DECK_2: f32 = DECK_1 + DECK_HEIGHT;
const ROCKET_CEILING: f32 = DECK_2 + DECK_HEIGHT;

const LOWER_LADDER_X: f32 = 400.0;
const UPPER_LADDER_X: f32 = -400.0;
const LADDER_GAP: f32 = LADDER_CLEARANCE;
const AIRLOCK_X: f32 = HULL_RIGHT - WALL_THICKNESS / 2.0;

const fn plate(from: f32, to: f32, top: f32) -> Platform {
    Platform::with_top((from + to) / 2.0, top, to - from)
}

const ROCKET_PLATFORMS: [Platform; 6] = [
    plate(HULL_LEFT, HULL_RIGHT, DECK_0),
    plate(HULL_LEFT, LOWER_LADDER_X - LADDER_GAP / 2.0, DECK_1),
    plate(LOWER_LADDER_X + LADDER_GAP / 2.0, HULL_RIGHT, DECK_1),
    plate(HULL_LEFT, UPPER_LADDER_X - LADDER_GAP / 2.0, DECK_2),
    plate(UPPER_LADDER_X + LADDER_GAP / 2.0, HULL_RIGHT, DECK_2),
    plate(HULL_LEFT, HULL_RIGHT, ROCKET_CEILING),
];

const DECK_0_DOOR: Door = Door::bulkhead(BULKHEAD_X, DECK_0);
const DECK_1_DOOR: Door = Door::bulkhead(BULKHEAD_X, DECK_1);
const DECK_2_DOOR: Door = Door::bulkhead(BULKHEAD_X, DECK_2);
const AIRLOCK: Door = Door::airlock(AIRLOCK_X, DECK_2);

const ROCKET_DOORS: [Door; 4] = [DECK_0_DOOR, DECK_1_DOOR, DECK_2_DOOR, AIRLOCK];

const ROCKET_WALLS: [Wall; 5] = [
    Wall::between(HULL_LEFT, DECK_0, ROCKET_CEILING),
    Wall::between(HULL_RIGHT, DECK_0, ROCKET_CEILING),
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

const ROCKET_CRATES: [Vec2; 4] = [
    Vec2::new(-300.0, DECK_0 + 140.0),
    Vec2::new(200.0, DECK_1 + 140.0),
    Vec2::new(-180.0, DECK_2 + 140.0),
    Vec2::new(-520.0, DECK_2 + 140.0),
];

const ROCKET_SPAWN: Vec2 = Vec2::new(HULL_LEFT + 120.0, DECK_0 + 60.0);

const ROCKET_CONFIG: LevelConfig = LevelConfig {
    platforms: &ROCKET_PLATFORMS,
    walls: &ROCKET_WALLS,
    ladders: &ROCKET_LADDERS,
    doors: &ROCKET_DOORS,
    crates: &ROCKET_CRATES,
    player_spawn: ROCKET_SPAWN,
    camera: CameraMode::Follow {
        zoom: INTERIOR_ZOOM,
        bounds: Rect::new(
            HULL_LEFT - WALL_THICKNESS,
            DECK_0 - PLATFORM_HEIGHT,
            HULL_RIGHT + WALL_THICKNESS,
            ROCKET_CEILING,
        ),
    },
    portal_positions: &[],
    portal_minigames: &[],
};

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

/// How high above the floor of its room a breach hangs.
///
/// Low enough that a player walking the deck passes through it — a portal is
/// walked into rather than worked, and one hung at head height would be strolled
/// under and left behind, which on the rocket would mean an airlock that never
/// unlocks. See `a_portal_in_a_room_is_walked_into`.
const PORTAL_MOUNT_HEIGHT: f32 = 56.0;

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

    /// The centre of a breach opened in this room. Over the same clear stretch
    /// of wall the panel is bolted to, which is safe because no two puzzles are
    /// ever dealt the same room — see [`crate::puzzles::RocketPuzzles`].
    pub const fn portal_mount(self) -> Vec2 {
        let at = self.fixture();

        Vec2::new(at.x, at.y + PORTAL_MOUNT_HEIGHT)
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

const ASCENT_PLATFORMS: [Platform; 13] = [
    Platform::with_top(0.0, GROUND_TOP, 4600.0),
    Platform::with_top(-1600.0, GROUND_TOP + 180.0, 330.0),
    Platform::with_top(-1240.0, GROUND_TOP + 280.0, 330.0),
    Platform::with_top(-880.0, GROUND_TOP + 210.0, 330.0),
    Platform::with_top(-520.0, GROUND_TOP + 320.0, 330.0),
    Platform::with_top(-160.0, GROUND_TOP + 240.0, 330.0),
    Platform::with_top(200.0, GROUND_TOP + 350.0, 330.0),
    Platform::with_top(560.0, GROUND_TOP + 260.0, 330.0),
    Platform::with_top(920.0, GROUND_TOP + 360.0, 330.0),
    Platform::with_top(1300.0, GROUND_TOP + 480.0, 220.0),
    Platform::with_top(1520.0, GROUND_TOP + 610.0, 220.0),
    Platform::with_top(1740.0, GROUND_TOP + 740.0, 220.0),
    Platform::with_top(1960.0, GROUND_TOP + 870.0, 220.0),
];

const ASCENT_CRATES: [Vec2; 5] = [
    Vec2::new(-1240.0, GROUND_TOP + 400.0),
    Vec2::new(-520.0, GROUND_TOP + 440.0),
    Vec2::new(200.0, GROUND_TOP + 470.0),
    Vec2::new(920.0, GROUND_TOP + 480.0),
    Vec2::new(1740.0, GROUND_TOP + 860.0),
];

const ASCENT_EXIT_X: f32 = 2048.0;
const ASCENT_EXIT: Door = Door::airlock(ASCENT_EXIT_X, GROUND_TOP + 870.0);

const EMPTY_WALLS: [Wall; 0] = [];
const EMPTY_LADDERS: [Ladder; 0] = [];

const ASCENT_DOORS: [Door; 1] = [ASCENT_EXIT];

const ASCENT_PORTALS: [Vec2; 3] = [
    Vec2::new(-1600.0, GROUND_TOP + 180.0 + 48.0),
    Vec2::new(-880.0, GROUND_TOP + 210.0 + 48.0),
    Vec2::new(560.0, GROUND_TOP + 260.0 + 48.0),
];

const ASCENT_PORTAL_MINIGAMES: [MinigameId; 2] = [MinigameId::TapChallenge, MinigameId::BrokenWire];

const ASCENT_CONFIG: LevelConfig = LevelConfig {
    platforms: &ASCENT_PLATFORMS,
    walls: &EMPTY_WALLS,
    ladders: &EMPTY_LADDERS,
    doors: &ASCENT_DOORS,
    crates: &ASCENT_CRATES,
    player_spawn: Vec2::new(-2050.0, GROUND_TOP + 60.0),
    camera: CameraMode::Follow {
        zoom: FOLLOW_ZOOM,
        bounds: FOLLOW_BOUNDS,
    },
    portal_positions: &ASCENT_PORTALS,
    portal_minigames: &ASCENT_PORTAL_MINIGAMES,
};

// ---------------------------------------------------------------------------
// Upper deck level
// ---------------------------------------------------------------------------

const UPPER_DECK_PLATFORMS: [Platform; 13] = [
    Platform::with_top(0.0, GROUND_TOP, 4600.0),
    Platform::with_top(-1480.0, GROUND_TOP + 220.0, 250.0),
    Platform::with_top(-1160.0, GROUND_TOP + 310.0, 290.0),
    Platform::with_top(-840.0, GROUND_TOP + 200.0, 260.0),
    Platform::with_top(-520.0, GROUND_TOP + 380.0, 250.0),
    Platform::with_top(-200.0, GROUND_TOP + 270.0, 310.0),
    Platform::with_top(120.0, GROUND_TOP + 420.0, 280.0),
    Platform::with_top(440.0, GROUND_TOP + 310.0, 300.0),
    Platform::with_top(760.0, GROUND_TOP + 460.0, 260.0),
    Platform::with_top(1120.0, GROUND_TOP + 520.0, 210.0),
    Platform::with_top(1320.0, GROUND_TOP + 640.0, 210.0),
    Platform::with_top(1520.0, GROUND_TOP + 760.0, 210.0),
    Platform::with_top(1980.0, GROUND_TOP + 880.0, 240.0),
];

const UPPER_DECK_CRATES: [Vec2; 5] = [
    Vec2::new(-1160.0, UPPER_DECK_PLATFORMS[2].top() + 120.0),
    Vec2::new(-200.0, UPPER_DECK_PLATFORMS[5].top() + 120.0),
    Vec2::new(440.0, UPPER_DECK_PLATFORMS[7].top() + 120.0),
    Vec2::new(1320.0, UPPER_DECK_PLATFORMS[10].top() + 120.0),
    Vec2::new(1720.0, UPPER_DECK_PLATFORMS[12].top() + 120.0),
];

const UPPER_DECK_PORTALS: [Vec2; 4] = [
    Vec2::new(-1480.0, UPPER_DECK_PLATFORMS[1].top() + 48.0),
    Vec2::new(-840.0, UPPER_DECK_PLATFORMS[3].top() + 48.0),
    Vec2::new(120.0, UPPER_DECK_PLATFORMS[6].top() + 48.0),
    Vec2::new(760.0, UPPER_DECK_PLATFORMS[8].top() + 48.0),
];

const UPPER_DECK_EXIT_X: f32 = 2048.0;
const UPPER_DECK_EXIT: Door = Door::airlock(UPPER_DECK_EXIT_X, UPPER_DECK_PLATFORMS[12].top());

const UPPER_DECK_DOORS: [Door; 1] = [UPPER_DECK_EXIT];

const UPPER_DECK_CONFIG: LevelConfig = LevelConfig {
    platforms: &UPPER_DECK_PLATFORMS,
    walls: &EMPTY_WALLS,
    ladders: &EMPTY_LADDERS,
    doors: &UPPER_DECK_DOORS,
    crates: &UPPER_DECK_CRATES,
    player_spawn: Vec2::new(-2000.0, GROUND_TOP + 60.0),
    camera: CameraMode::Follow {
        zoom: FOLLOW_ZOOM,
        bounds: FOLLOW_BOUNDS,
    },
    portal_positions: &UPPER_DECK_PORTALS,
    portal_minigames: &ASCENT_PORTAL_MINIGAMES,
};

impl Level {
    pub fn config(self) -> LevelConfig {
        match self {
            Level::Rocket => ROCKET_CONFIG,
            Level::Ascent => ASCENT_CONFIG,
            Level::UpperDeck => UPPER_DECK_CONFIG,
        }
    }

    pub fn next(self) -> Option<Self> {
        match self {
            Level::Rocket => Some(Level::Ascent),
            Level::Ascent => Some(Level::UpperDeck),
            Level::UpperDeck => None,
        }
    }

    pub fn platforms(self) -> &'static [Platform] {
        self.config().platforms
    }

    pub fn walls(self) -> &'static [Wall] {
        self.config().walls
    }

    pub fn ladders(self) -> &'static [Ladder] {
        self.config().ladders
    }

    pub fn doors(self) -> &'static [Door] {
        self.config().doors
    }

    /// The rooms the level is divided into. Only the rocket has any: the ascent
    /// is open ground, with nothing to be inside of.
    pub fn rooms(self) -> &'static [Room] {
        match self {
            Level::Rocket => &ROCKET_ROOMS,
            Level::Ascent => &[],
            Level::UpperDeck => &[],
        }
    }

    pub fn crates(self) -> &'static [Vec2] {
        self.config().crates
    }

    pub fn player_spawn(self) -> Vec2 {
        self.config().player_spawn
    }

    /// Where this level's portals are written down. The rocket's are not: they
    /// stand in the rooms dealt for the run, so it is
    /// [`crate::puzzles::RocketPuzzles`] that says where they are.
    pub fn portals(self) -> &'static [Vec2] {
        self.config().portal_positions
    }

    pub fn portal_minigames(self) -> &'static [MinigameId] {
        self.config().portal_minigames
    }

    /// How many portals this level puts up, which is what the objective counts
    /// down. Inside the rocket that is one breach per kind of challenge, one
    /// room each, however the rooms happen to be dealt.
    pub fn portal_count(self) -> usize {
        match self {
            Level::Rocket => MINIGAME_COUNT,
            _ => self.portals().len(),
        }
    }

    #[allow(dead_code)]
    pub fn portal_anchor(self) -> Option<Vec2> {
        self.portals().first().copied()
    }

    pub fn camera(self) -> CameraMode {
        self.config().camera
    }
}

/// Inserted rather than assigned, so change detection fires — and the camera
/// re-frames — every time a run starts.
pub fn reset_level(mut commands: Commands) {
    commands.insert_resource(Level::default());
}

/// Marks a queued level transition that should be applied when gameplay returns
/// to the running state.
#[derive(Resource, Debug, Clone, Copy)]
pub struct PendingLevelAdvance(pub Level);

/// Routes minigame outcomes through the level, which is where branching rules
/// belong once there is more than one level and more than one challenge.
pub fn react_to_minigame_result(
    mut commands: Commands,
    completed: Option<Res<CompletedMinigame>>,
    triggered_portal: Option<Res<TriggeredPortal>>,
    mut progress: ResMut<LevelProgress>,
    mut next_playing: ResMut<NextState<PlayingState>>,
) {
    let Some(completed) = completed else {
        return;
    };

    match (completed.id, completed.outcome) {
        (_, MinigameOutcome::Success) => {
            progress.complete_portal();
            if let Some(triggered_portal) = triggered_portal {
                commands.entity(triggered_portal.0).despawn();
            }
        }
        (_, MinigameOutcome::Failure) | (_, MinigameOutcome::TimedOut) => {
            next_playing.set(PlayingState::GameOver);
        }
    }

    commands.remove_resource::<TriggeredPortal>();
    commands.remove_resource::<CompletedMinigame>();
}

#[cfg(test)]
mod tests {
    use bevy::ecs::schedule::IntoScheduleConfigs;
    use bevy::prelude::*;

    use super::{
        AIRLOCK, ASCENT_PLATFORMS, BULKHEAD_X, CameraMode, DECK_0, DECK_1, DECK_2, Door,
        FOLLOW_ZOOM, GROUND_TOP, LADDER_GAP, LOWER_LADDER_X, Level, LevelProgress, PLATFORM_HEIGHT,
        ROOM_COUNT, Room, UPPER_LADDER_X,
    };
    use crate::config::{PLAYER_HEIGHT, VIEW_HEIGHT};

    #[test]
    fn a_run_opens_inside_the_rocket() {
        assert_eq!(Level::default(), Level::Rocket);
    }

    #[test]
    fn levels_chain_in_order() {
        assert_eq!(Level::Rocket.next(), Some(Level::Ascent));
        assert_eq!(Level::Ascent.next(), Some(Level::UpperDeck));
        assert_eq!(Level::UpperDeck.next(), None);
    }

    #[test]
    fn only_rocket_has_structural_geometry() {
        assert!(!Level::Rocket.walls().is_empty());
        assert!(!Level::Rocket.ladders().is_empty());
        assert!(!Level::Rocket.doors().is_empty());

        assert!(Level::Ascent.walls().is_empty());
        assert!(Level::Ascent.ladders().is_empty());
        assert_eq!(Level::Ascent.doors().len(), 1);

        assert!(Level::UpperDeck.walls().is_empty());
        assert!(Level::UpperDeck.ladders().is_empty());
        assert_eq!(Level::UpperDeck.doors().len(), 1);
    }

    #[test]
    fn ascent_has_a_final_exit_door() {
        assert_eq!(
            Level::Ascent.doors()[0].kind,
            crate::door::DoorKind::Airlock
        );
    }

    #[test]
    fn upper_deck_has_a_final_exit_door() {
        assert_eq!(
            Level::UpperDeck.doors()[0].kind,
            crate::door::DoorKind::Airlock
        );
    }

    /// The rocket puts up one breach per kind of challenge, so a run works
    /// every puzzle the game has before it ever reaches the pad.
    #[test]
    fn the_rocket_carries_one_of_every_challenge() {
        use crate::minigames::MINIGAME_COUNT;

        assert_eq!(Level::Rocket.portal_count(), MINIGAME_COUNT);
        // Its portals stand in the rooms dealt for the run rather than at
        // positions written into the layout.
        assert!(Level::Rocket.portals().is_empty());
    }

    /// A breach is walked into rather than worked, so one hung out of the
    /// player's way would be strolled past — and the airlock waits on it, so
    /// that is a run that cannot be finished.
    #[test]
    fn a_portal_in_a_room_is_walked_into() {
        use crate::config::PLAYER_HEIGHT;
        use crate::portal::PORTAL_RADIUS;

        for index in 0..ROOM_COUNT {
            let room = Room::from_index(index);
            let breach = room.portal_mount();
            let walking_past = Vec2::new(breach.x, room.floor() + PLAYER_HEIGHT / 2.0);

            assert!(
                walking_past.distance(breach) < PORTAL_RADIUS,
                "the breach in {} is hung clear of a player walking under it",
                room.label()
            );
        }
    }

    /// The other fixtures in a room: a breach must not be opened over a ladder
    /// or across a doorway, where it would take the player every time they went
    /// to use either.
    #[test]
    fn a_portal_is_clear_of_the_ladders_and_the_doors() {
        use crate::portal::PORTAL_RADIUS;

        for index in 0..ROOM_COUNT {
            let room = Room::from_index(index);
            let breach = room.portal_mount();

            for ladder in Level::Rocket.ladders() {
                let column = ladder.reach();
                let clear = breach.x + PORTAL_RADIUS <= column.min.x
                    || breach.x - PORTAL_RADIUS >= column.max.x;

                assert!(
                    clear,
                    "the breach in {} is opened over the ladder at x={}",
                    room.label(),
                    ladder.x
                );
            }

            for door in Level::Rocket.doors() {
                assert!(
                    (breach.x - door.at.x).abs() > PORTAL_RADIUS,
                    "the breach in {} is opened across {door:?}",
                    room.label()
                );
            }
        }
    }

    #[test]
    fn the_outdoor_levels_have_portals() {
        assert!(Level::Ascent.portal_anchor().is_some());
        assert!(Level::Ascent.portals().len() > 1);
        assert!(Level::Ascent.portal_minigames().len() > 1);

        assert!(Level::UpperDeck.portal_anchor().is_some());
        assert!(Level::UpperDeck.portals().len() > 1);
        assert!(Level::UpperDeck.portal_minigames().len() > 1);
    }

    #[test]
    fn second_level_has_distinct_layout_data() {
        let ascent = Level::Ascent.config();
        let upper = Level::UpperDeck.config();

        assert_ne!(ascent.platforms.as_ptr(), upper.platforms.as_ptr());
        assert_ne!(ascent.crates.as_ptr(), upper.crates.as_ptr());
        assert_ne!(
            Level::Ascent.player_spawn(),
            Level::UpperDeck.player_spawn()
        );
    }

    #[test]
    fn every_level_spawns_inside_camera_bounds() {
        for level in [Level::Rocket, Level::Ascent, Level::UpperDeck] {
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
        use crate::door::{leave_through_airlock, sync_airlock_lock_state, use_doors};
        use crate::ladder::climb_ladder;
        use crate::panel::Panel;
        use crate::physics::configure_physics;
        use crate::player::{Player, jump, move_player, probe_ground};
        use crate::portal::{Portal, enter_portal};
        use crate::puzzles::RocketPuzzles;
        use crate::setup::{RunState, build_level};
        use crate::state::{GameState, PlayingState};
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
                    bevy::state::app::StatesPlugin,
                ));
                app.init_state::<GameState>();
                app.add_sub_state::<PlayingState>();
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
                app.init_resource::<RocketPuzzles>();
                app.insert_resource(LevelProgress::new(Level::Rocket));
                app.add_systems(
                    Startup,
                    |mut commands: Commands,
                     assets: Res<AssetServer>,
                     mut images: ResMut<Assets<Image>>,
                     panel: Res<Panel>,
                     puzzles: Res<RocketPuzzles>,
                     progress: Res<LevelProgress>| {
                        build_level(
                            &mut commands,
                            &assets,
                            &mut images,
                            Level::Rocket,
                            RunState {
                                puzzles: *puzzles,
                                panel: *panel,
                                progress: *progress,
                            },
                        );
                    },
                );
                app.add_systems(
                    Update,
                    (
                        sync_airlock_lock_state,
                        move_player,
                        probe_ground,
                        climb_ladder,
                        jump,
                        // The breaches are walked into rather than worked, so
                        // this is what says the crossing meets them at all.
                        enter_portal,
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

            /// How many of the rocket's breaches the crossing has walked into.
            fn breaches_met(&mut self) -> usize {
                let mut query = self.app.world_mut().query::<&Portal>();

                query
                    .iter(self.app.world())
                    .filter(|portal| portal.used)
                    .count()
            }

            /// Signs off the work the geometry cannot do on its own: the
            /// minigame overlays are not in this app, so the breaches walked
            /// into are booked in here.
            fn sign_off_the_repairs(&mut self) {
                self.app.world_mut().resource_mut::<Panel>().solved = true;
                let mut progress = self.app.world_mut().resource_mut::<LevelProgress>();
                progress.completed_portals = progress.total_portals;
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

            // The route to the airlock walks every room, so it cannot have gone
            // past a breach without meeting it — a breach strolled past is an
            // airlock that never unlocks.
            assert_eq!(
                run.breaches_met(),
                Level::Rocket.portal_count(),
                "the crossing walked past a breach without triggering it"
            );

            // The final airlock only opens after every job on the level is done.
            run.sign_off_the_repairs();
            run.step(&[]);

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

    /// Inside the rocket every kind of job has to be done: the panel *and* each
    /// breach. Out on the open levels there is no panel to ask for.
    #[test]
    fn all_obstacles_require_panel_only_when_present() {
        let rocket_progress = LevelProgress::new(Level::Rocket);
        let rocket_room = Level::Rocket.rooms()[0];

        assert!(!rocket_progress.all_obstacles_completed(Level::Rocket, rocket_room, false));
        assert!(
            !rocket_progress.all_obstacles_completed(Level::Rocket, rocket_room, true),
            "the panel alone opened the airlock with breaches still outstanding"
        );

        let mut rocket_done = LevelProgress::new(Level::Rocket);
        rocket_done.completed_portals = rocket_done.total_portals;
        assert!(!rocket_done.all_obstacles_completed(Level::Rocket, rocket_room, false));
        assert!(rocket_done.all_obstacles_completed(Level::Rocket, rocket_room, true));

        let ascent_progress = LevelProgress::new(Level::Ascent);
        assert!(!ascent_progress.all_obstacles_completed(Level::Ascent, rocket_room, false));

        let mut ascent_done = LevelProgress::new(Level::Ascent);
        ascent_done.completed_portals = ascent_done.total_portals;
        assert!(ascent_done.all_obstacles_completed(Level::Ascent, rocket_room, false));
    }
}
