//! The level a run is made of, and what makes it what it is.
//!
//! Boarding the rocket drops the player straight into the ascent: open ground
//! with no walls at all, framed by a camera zoomed in on the player that pans
//! along as they move. The mission clock runs for as long as the run does.

use bevy::prelude::*;

use crate::config::{DESIGN_HEIGHT, FOLLOW_ZOOM, PLATFORM_HEIGHT};
use crate::minigames::{CompletedMinigame, MinigameConfig, MinigameId, MinigameOutcome};
use crate::platform::Platform;
use crate::state::PlayingState;

/// Marks the level geometry: platforms, crates and the player. Cleared when the
/// run ends, which is what separates it from the HUD's
/// [`crate::setup::GameEntity`].
#[derive(Component, Clone)]
pub struct LevelEntity;

/// Which scene the current run is in. There are two for now, but the camera and
/// the geometry are still asked for through it, so adding another is a matter of
/// adding a variant rather than unpicking the systems.
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    /// Open ground. No walls, and the camera zooms in and tracks the player
    /// instead of keeping the level still.
    #[default]
    Ascent,
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

#[derive(Debug, Clone, Copy)]
pub struct LevelConfig {
    pub platforms: &'static [Platform],
    pub crates: &'static [Vec2],
    pub player_spawn: Vec2,
    pub camera: CameraMode,
    pub minigame: MinigameConfig,
    pub portal_ahead: f32,
    pub portal_up: f32,
    pub portal_camera_inset: f32,
}

/// Top of the level ground.
const GROUND_TOP: f32 = -DESIGN_HEIGHT / 2.0;

/// Shared camera bounds for the player-following levels.
const FOLLOW_BOUNDS: Rect = Rect::new(-2100.0, GROUND_TOP - PLATFORM_HEIGHT, 2100.0, GROUND_TOP + 1400.0);

/// The original ascent layout, kept exactly as it was before the cleanup.
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

const ASCENT_CONFIG: LevelConfig = LevelConfig {
    platforms: &ASCENT_PLATFORMS,
    crates: &ASCENT_CRATES,
    player_spawn: Vec2::new(-2050.0, GROUND_TOP + 60.0),
    camera: CameraMode::Follow {
        zoom: FOLLOW_ZOOM,
        bounds: FOLLOW_BOUNDS,
    },
    minigame: MinigameConfig {
        id: MinigameId::TapChallenge,
        time_limit_seconds: 8.0,
    },
    portal_ahead: 110.0,
    portal_up: 48.0,
    portal_camera_inset: 20.0,
};

/// The second level keeps the same flow, but with a different climb rhythm.
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
    Platform::with_top(1720.0, GROUND_TOP + 880.0, 210.0),
];

const UPPER_DECK_CRATES: [Vec2; 5] = [
    Vec2::new(-1160.0, UPPER_DECK_PLATFORMS[2].top() + 120.0),
    Vec2::new(-200.0, UPPER_DECK_PLATFORMS[5].top() + 120.0),
    Vec2::new(440.0, UPPER_DECK_PLATFORMS[7].top() + 120.0),
    Vec2::new(1320.0, UPPER_DECK_PLATFORMS[10].top() + 120.0),
    Vec2::new(1720.0, UPPER_DECK_PLATFORMS[12].top() + 120.0),
];

const UPPER_DECK_CONFIG: LevelConfig = LevelConfig {
    platforms: &UPPER_DECK_PLATFORMS,
    crates: &UPPER_DECK_CRATES,
    player_spawn: Vec2::new(-2000.0, GROUND_TOP + 60.0),
    camera: CameraMode::Follow {
        zoom: FOLLOW_ZOOM,
        bounds: FOLLOW_BOUNDS,
    },
    minigame: MinigameConfig {
        id: MinigameId::TapChallenge,
        time_limit_seconds: 8.0,
    },
    portal_ahead: 110.0,
    portal_up: 48.0,
    portal_camera_inset: 20.0,
};

impl Level {
    pub fn config(self) -> LevelConfig {
        match self {
            Level::Ascent => ASCENT_CONFIG,
            Level::UpperDeck => UPPER_DECK_CONFIG,
        }
    }

    pub fn platforms(self) -> &'static [Platform] {
        self.config().platforms
    }

    pub fn crates(self) -> &'static [Vec2] {
        self.config().crates
    }

    pub fn player_spawn(self) -> Vec2 {
        self.config().player_spawn
    }

    pub fn minigame(self) -> MinigameConfig {
        self.config().minigame
    }

    pub fn portal_anchor(self) -> Vec2 {
        let config = self.config();
        let last = &config.platforms[config.platforms.len() - 1];
        let desired_x = last.centre.x + last.width / 2.0 + config.portal_ahead;
        let visible_limit_x = 2100.0 - config.portal_camera_inset;
        Vec2::new(desired_x.min(visible_limit_x), last.top() + config.portal_up)
    }

    pub fn camera(self) -> CameraMode {
        self.config().camera
    }

    pub fn next(self) -> Option<Self> {
        match self {
            Level::Ascent => Some(Level::UpperDeck),
            Level::UpperDeck => None,
        }
    }
}

/// Inserted rather than assigned, so change detection fires — and the camera
/// re-frames — every time a run starts.
pub fn reset_level(mut commands: Commands) {
    commands.insert_resource(Level::default());
}

/// Marks a queued level transition that should be applied when gameplay
/// returns to the running state.
#[derive(Resource, Debug, Clone, Copy)]
pub struct PendingLevelAdvance(pub Level);

/// Routes minigame outcomes through the level, which is where branching rules
/// belong once there is more than one level and more than one challenge.
pub fn react_to_minigame_result(
    mut commands: Commands,
    completed: Option<Res<CompletedMinigame>>,
    mut next_playing: ResMut<NextState<PlayingState>>,
) {
    let Some(completed) = completed else {
        return;
    };

    match (completed.id, completed.outcome) {
        (MinigameId::TapChallenge, MinigameOutcome::Success) => {
            next_playing.set(PlayingState::MissionComplete);
        }
        (MinigameId::TapChallenge, MinigameOutcome::Failure)
        | (MinigameId::TapChallenge, MinigameOutcome::TimedOut) => {
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
    fn a_run_opens_on_the_ascent() {
        assert_eq!(Level::default(), Level::Ascent);
    }

    #[test]
    fn the_levels_chain_in_order() {
        assert_eq!(Level::Ascent.next(), Some(Level::UpperDeck));
        assert_eq!(Level::UpperDeck.next(), None);
    }

    #[test]
    fn the_ascent_tracks_the_player() {
        assert!(matches!(
            Level::Ascent.camera(),
            CameraMode::Follow { zoom, .. } if zoom > 1.0
        ));
    }

    #[test]
    fn the_player_spawns_inside_the_camera_bounds() {
        let CameraMode::Follow { bounds, .. } = Level::Ascent.camera() else {
            panic!("the ascent is meant to use a following camera");
        };

        assert!(bounds.contains(Level::Ascent.player_spawn()));
    }

    #[test]
    fn the_second_level_has_its_own_layout() {
        let ascent = Level::Ascent.config();
        let upper = Level::UpperDeck.config();

        assert_ne!(ascent.platforms.as_ptr(), upper.platforms.as_ptr());
        assert_ne!(ascent.crates.as_ptr(), upper.crates.as_ptr());
        assert_ne!(Level::Ascent.player_spawn(), Level::UpperDeck.player_spawn());
    }

    #[test]
    fn the_second_level_spawns_within_camera_bounds() {
        let CameraMode::Follow { bounds, .. } = Level::UpperDeck.camera() else {
            panic!("the upper deck is meant to use a following camera");
        };

        assert!(bounds.contains(Level::UpperDeck.player_spawn()));
        assert!(Level::UpperDeck.player_spawn().x < 0.0);
        assert!(Level::UpperDeck.player_spawn().y < 0.0 + PLAYER_HEIGHT);
    }
}
