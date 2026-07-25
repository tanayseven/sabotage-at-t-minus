//! The levels a run is made of, and what makes each one different.
//!
//! A run starts inside the rocket. Leaving through the airlock transitions to
//! the ascent, and then to the upper deck through the portal/minigame flow.

use bevy::prelude::*;

use crate::config::{DESIGN_HEIGHT, FOLLOW_ZOOM, INTERIOR_ZOOM, PLATFORM_HEIGHT, WALL_THICKNESS};
use crate::door::Door;
use crate::ladder::{LADDER_CLEARANCE, Ladder};
use crate::minigames::{CompletedMinigame, MinigameId, MinigameOutcome};
use crate::platform::Platform;
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
            total_portals: level.portals().len(),
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
// Ascent level
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

const ASCENT_PORTAL_MINIGAMES: [MinigameId; 2] = [
    MinigameId::TapChallenge,
    MinigameId::SequenceChallenge,
];

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

    pub fn crates(self) -> &'static [Vec2] {
        self.config().crates
    }

    pub fn player_spawn(self) -> Vec2 {
        self.config().player_spawn
    }

    pub fn portals(self) -> &'static [Vec2] {
        self.config().portal_positions
    }

    pub fn portal_minigames(self) -> &'static [MinigameId] {
        self.config().portal_minigames
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
    mut progress: ResMut<LevelProgress>,
    mut doors: Query<(&mut Door, &mut Sprite)>,
    mut next_playing: ResMut<NextState<PlayingState>>,
) {
    let Some(completed) = completed else {
        return;
    };

    match (completed.id, completed.outcome) {
        (_, MinigameOutcome::Success) => {
            if progress.complete_portal() {
                for (mut door, mut sprite) in &mut doors {
                    if door.kind == crate::door::DoorKind::Airlock {
                        door.locked = false;
                        sprite.color = door.color();
                    }
                }
            }
        }
        (_, MinigameOutcome::Failure) | (_, MinigameOutcome::TimedOut) => {
            next_playing.set(PlayingState::GameOver);
        }
    }

    commands.remove_resource::<CompletedMinigame>();
}

#[cfg(test)]
mod tests {
    use super::{CameraMode, Level};
    use crate::config::PLAYER_HEIGHT;

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
        assert_eq!(Level::Ascent.doors()[0].kind, crate::door::DoorKind::Airlock);
    }

    #[test]
    fn upper_deck_has_a_final_exit_door() {
        assert_eq!(Level::UpperDeck.doors()[0].kind, crate::door::DoorKind::Airlock);
    }

    #[test]
    fn the_outdoor_levels_have_portals() {
        assert!(Level::Rocket.portal_anchor().is_none());

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
        assert_ne!(Level::Ascent.player_spawn(), Level::UpperDeck.player_spawn());
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
    fn upper_deck_spawn_stays_on_the_left_side() {
        assert!(Level::UpperDeck.player_spawn().x < 0.0);
        assert!(Level::UpperDeck.player_spawn().y < PLAYER_HEIGHT);
    }
}
