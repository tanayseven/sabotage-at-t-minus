//! The levels a run is made of, and what makes each one different.
//!
//! A run starts in the rocket bay, which is boxed in by the walls and fits on
//! screen whole. Past it lies the ascent: open ground with no walls at all,
//! framed by a camera zoomed in on the player that pans along as they move.
//! Walking into a level's exit pad swaps the geometry for the next level's
//! without ending the run — the HUD and the mission clock carry straight over.

use bevy::prelude::*;

use crate::config::{DESIGN_HEIGHT, FOLLOW_ZOOM, PLATFORM_HEIGHT, PLAYER_HEIGHT, PLAYER_WIDTH};
use crate::platform::Platform;
use crate::player::Player;
use crate::setup::build_level;

/// Marks the level geometry: walls, platforms, crates, the exit pad and the
/// player. Cleared both when the run ends and when it moves on to the next
/// level, which is what separates it from the HUD's [`crate::setup::GameEntity`].
#[derive(Component, Clone)]
pub struct LevelEntity;

/// Which scene the current run is in. Levels run in the order below, and the
/// mission clock spans the whole run rather than restarting on each one.
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    /// Where the run starts: walled in on all four sides, and small enough to
    /// hold in frame whole.
    #[default]
    RocketBay,
    /// Open ground past the bay. No walls, and the camera zooms in and tracks
    /// the player instead of keeping the level still.
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

/// Top of the floor, shared by the bay's bottom wall and the ascent's ground.
const GROUND_TOP: f32 = -DESIGN_HEIGHT / 2.0;

const BAY_PLATFORMS: [Platform; 3] = [
    Platform::new(-380.0, -120.0, 360.0),
    Platform::new(40.0, 60.0, 300.0),
    Platform::new(430.0, -40.0, 260.0),
];

const BAY_CRATES: [Vec2; 4] = [
    Vec2::new(-300.0, 40.0),
    Vec2::new(120.0, 300.0),
    Vec2::new(200.0, 300.0),
    // Kept clear of the exit pad, so nothing settles on top of the way out.
    Vec2::new(380.0, 200.0),
];

/// The bay's exit pad, standing on the right-hand platform.
const BAY_EXIT: Vec2 = Vec2::new(470.0, BAY_PLATFORMS[2].top() + EXIT_SIZE.y / 2.0);

/// How far the ascent reaches either side of the origin — the line the camera
/// stops panning at, not a wall. Nothing stops the player walking past it.
const ASCENT_REACH: f32 = 2100.0;
/// Headroom above the ground for the camera to climb into, with room to spare
/// over the highest ledge so the top of the climb is not framed against nothing.
const ASCENT_CEILING: f32 = GROUND_TOP + 1400.0;

/// The ascent has to be laid out for a camera that only ever shows a slice of
/// it: at [`FOLLOW_ZOOM`] roughly 710x400 units are on screen at once, a third
/// of what the bay gets. Two rules fall out of that, and both were learned the
/// hard way from a layout drawn for the unzoomed view.
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
            Level::RocketBay => Some(Level::Ascent),
            Level::Ascent => None,
        }
    }

    /// Whether the level is boxed in by the four boundary walls. The ascent is
    /// deliberately not, which is what makes it feel like open ground.
    pub fn is_walled(self) -> bool {
        matches!(self, Level::RocketBay)
    }

    pub fn platforms(self) -> &'static [Platform] {
        match self {
            Level::RocketBay => &BAY_PLATFORMS,
            Level::Ascent => &ASCENT_PLATFORMS,
        }
    }

    pub fn crates(self) -> &'static [Vec2] {
        match self {
            Level::RocketBay => &BAY_CRATES,
            Level::Ascent => &ASCENT_CRATES,
        }
    }

    pub fn player_spawn(self) -> Vec2 {
        match self {
            Level::RocketBay => Vec2::new(-380.0, 260.0),
            Level::Ascent => Vec2::new(ASCENT_BAND_START - 450.0, GROUND_TOP + 60.0),
        }
    }

    /// Where the pad through to the next level stands, if there is one.
    pub fn exit(self) -> Option<Vec2> {
        match self {
            Level::RocketBay => Some(BAY_EXIT),
            Level::Ascent => None,
        }
    }

    pub fn camera(self) -> CameraMode {
        match self {
            Level::RocketBay => CameraMode::Fixed,
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

const EXIT_SIZE: Vec2 = Vec2::new(56.0, 96.0);
const EXIT_COLOR: Color = Color::srgb(0.45, 0.9, 0.55);
/// Behind the player, so walking into the pad reads as stepping through it.
const EXIT_Z: f32 = -2.0;

/// How far off the pad's centre still counts as walking into it: the player's
/// own half-width plus the pad's, which is exactly when the two overlap.
const EXIT_REACH: Vec2 = Vec2::new(
    (EXIT_SIZE.x + PLAYER_WIDTH) / 2.0,
    (EXIT_SIZE.y + PLAYER_HEIGHT) / 2.0,
);

pub fn spawn_exit(commands: &mut Commands, position: Vec2) {
    commands.spawn((
        LevelEntity,
        Sprite {
            color: EXIT_COLOR,
            custom_size: Some(EXIT_SIZE),
            ..default()
        },
        Transform::from_xyz(position.x, position.y, EXIT_Z),
    ));
}

/// Swaps the level geometry out from under the player the moment they step into
/// the exit pad. Overlap is measured off the transforms rather than a sensor
/// collider, the same way boarding the rocket is: a pad you can walk straight
/// through is not something the physics pipeline needs to know about.
pub fn reach_exit(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut level: ResMut<Level>,
    players: Query<&Transform, With<Player>>,
    built: Query<Entity, With<LevelEntity>>,
) {
    let Some(pad) = level.exit() else {
        return;
    };

    let reached = players.iter().any(|transform| {
        let offset = (transform.translation.truncate() - pad).abs();
        offset.x <= EXIT_REACH.x && offset.y <= EXIT_REACH.y
    });

    let Some(next) = reached.then(|| level.next()).flatten() else {
        return;
    };

    *level = next;

    for entity in &built {
        commands.entity(entity).despawn();
    }
    build_level(&mut commands, &assets, next);
}

#[cfg(test)]
mod tests {
    use super::{
        ASCENT_PLATFORMS, CameraMode, FOLLOW_ZOOM, GROUND_TOP, Level, PLATFORM_HEIGHT,
        PLAYER_HEIGHT,
    };

    #[test]
    fn the_run_ends_after_the_ascent() {
        assert_eq!(Level::default(), Level::RocketBay);
        assert_eq!(Level::RocketBay.next(), Some(Level::Ascent));
        assert_eq!(Level::Ascent.next(), None);
    }

    #[test]
    fn every_level_but_the_last_has_a_way_out() {
        assert!(Level::RocketBay.exit().is_some());
        assert!(Level::Ascent.exit().is_none());
    }

    #[test]
    fn the_ascent_is_open_and_tracks_the_player() {
        assert!(Level::RocketBay.is_walled());
        assert!(!Level::Ascent.is_walled());

        assert!(matches!(Level::RocketBay.camera(), CameraMode::Fixed));
        assert!(matches!(
            Level::Ascent.camera(),
            CameraMode::Follow { zoom, .. } if zoom > 1.0
        ));
    }

    /// A spawn outside the camera bounds would start the level with the player
    /// off screen, since the camera never pans past them.
    #[test]
    fn the_player_spawns_inside_the_camera_bounds() {
        let CameraMode::Follow { bounds, .. } = Level::Ascent.camera() else {
            panic!("the ascent is meant to use a following camera");
        };

        assert!(bounds.contains(Level::Ascent.player_spawn()));
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
