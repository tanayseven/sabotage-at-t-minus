//! The level a run is made of.
//!
//! A run is the rocket: six rooms with a job in three of them, and the airlock
//! at the far end of the top deck. The airlock is the way back out, and working
//! it is what finishes the run — there is nothing after the rocket, so what was
//! once a doorway onto the next level is now the end of the mission.

use bevy::prelude::*;

use crate::config::{DESIGN_HEIGHT, DESIGN_WIDTH, INTERIOR_ZOOM, PLATFORM_HEIGHT, WALL_THICKNESS};
use crate::door::Door;
use crate::ladder::{LADDER_CLEARANCE, Ladder};
use crate::minigames::{CompletedMinigame, MINIGAME_COUNT, MinigameOutcome};
use crate::platform::Platform;
use crate::portal::TriggeredPortal;
use crate::state::PlayingState;
use crate::wall::Wall;

/// Marks the level geometry: walls, platforms, ladders, doors, crates and the
/// player. Cleared when the run ends, which is what separates it from the HUD's
/// [`crate::setup::GameEntity`].
#[derive(Component, Clone)]
pub struct LevelEntity;

/// Which scene the current run is in. There is one, and it is kept as a level
/// rather than folded away so the camera, the HUD and the doors carry on asking
/// the level what to do rather than assuming it.
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    /// The whole run: the rooms inside the rocket.
    #[default]
    Rocket,
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

/// The layout of a level. The breaches are not in it: they stand in whichever
/// rooms the run dealt them, which is [`crate::puzzles::RocketPuzzles`]'s to say
/// rather than the layout's.
#[derive(Clone, Copy)]
pub struct LevelConfig {
    pub platforms: &'static [Platform],
    pub walls: &'static [Wall],
    pub ladders: &'static [Ladder],
    pub doors: &'static [Door],
    pub crates: &'static [Vec2],
    pub player_spawn: Vec2,
    pub camera: CameraMode,
}

/// How much of the run's breach objective has been sealed.
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

    /// The run is clear when every obstacle on the level has been worked: the
    /// panel (if this level has it) and every breach.
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

/// Top of the floor the rocket's bottom deck is laid on.
const GROUND_TOP: f32 = -DESIGN_HEIGHT / 2.0;

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
/// The way back out, at the far end of the top deck. Working it ends the run.
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

impl Level {
    pub fn config(self) -> LevelConfig {
        match self {
            Level::Rocket => ROCKET_CONFIG,
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

    /// The rooms the level is divided into.
    pub fn rooms(self) -> &'static [Room] {
        match self {
            Level::Rocket => &ROCKET_ROOMS,
        }
    }

    pub fn crates(self) -> &'static [Vec2] {
        self.config().crates
    }

    pub fn player_spawn(self) -> Vec2 {
        self.config().player_spawn
    }

    /// How many breaches this level puts up, which is what the objective counts
    /// down: one per kind of challenge, one room each, however the rooms happen
    /// to be dealt.
    pub fn portal_count(self) -> usize {
        match self {
            Level::Rocket => MINIGAME_COUNT,
        }
    }

    pub fn camera(self) -> CameraMode {
        self.config().camera
    }

    /// The stretch of level the hull lining is papered over. Taken from what the
    /// camera can reach rather than from the hull itself, so a pan that runs to
    /// the edge of the level shows plating rather than the void behind it.
    pub fn interior(self) -> Rect {
        match self.camera() {
            CameraMode::Follow { bounds, .. } => bounds,
            CameraMode::Fixed => {
                Rect::from_center_size(Vec2::ZERO, Vec2::new(DESIGN_WIDTH, DESIGN_HEIGHT))
            }
        }
    }
}

/// Inserted rather than assigned, so change detection fires — and the camera
/// re-frames — every time a run starts.
pub fn reset_level(mut commands: Commands) {
    commands.insert_resource(Level::default());
}

/// Routes minigame outcomes through the level, which is where the rules about
/// what a finished challenge means to the run belong.
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
        AIRLOCK, BULKHEAD_X, CameraMode, DECK_0, DECK_1, DECK_2, Door, LADDER_GAP, LOWER_LADDER_X,
        Level, LevelProgress, ROOM_COUNT, Room, UPPER_LADDER_X,
    };
    use crate::config::PLAYER_HEIGHT;

    #[test]
    fn a_run_opens_inside_the_rocket() {
        assert_eq!(Level::default(), Level::Rocket);
    }

    /// The rocket is the run: there is nothing after it, and the airlock at
    /// the end of the top deck is the way back out rather than a doorway onto
    /// somewhere else.
    #[test]
    fn the_rocket_is_the_whole_run() {
        assert!(!Level::Rocket.walls().is_empty());
        assert!(!Level::Rocket.ladders().is_empty());
        assert_eq!(Level::Rocket.doors().len(), 4);
        assert_eq!(AIRLOCK.kind, crate::door::DoorKind::Airlock);
    }

    /// The exit is on the far side of the top deck, past the last bulkhead —
    /// the end of the crossing, not somewhere the player starts next to. Read
    /// off the level's own doors, so a layout that stopped shipping the airlock
    /// fails here rather than passing against the constant.
    #[test]
    fn the_exit_is_at_the_far_end_of_the_rocket() {
        let spawn = Level::Rocket.player_spawn();
        let exit = Level::Rocket
            .doors()
            .iter()
            .find(|door| door.kind == crate::door::DoorKind::Airlock)
            .expect("the rocket has no way back out");

        assert!(exit.at.x > BULKHEAD_X, "the exit is not past the bulkhead");
        assert!(
            exit.sill() > spawn.y,
            "the exit is on the deck the run starts on"
        );
    }

    /// The rocket puts up one breach per kind of challenge, so a run works
    /// every puzzle the game has before it ever reaches the pad.
    #[test]
    fn the_rocket_carries_one_of_every_challenge() {
        use crate::minigames::MINIGAME_COUNT;

        assert_eq!(Level::Rocket.portal_count(), MINIGAME_COUNT);
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

    /// The run has to open with the whole of the drop point on screen.
    #[test]
    fn the_run_spawns_inside_the_camera_bounds() {
        let CameraMode::Follow { bounds, .. } = Level::Rocket.camera() else {
            panic!("the rocket is meant to use a following camera");
        };

        assert!(bounds.contains(Level::Rocket.player_spawn()));
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
                app.insert_state(GameState::Playing);
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
                     panel: Res<Panel>,
                     puzzles: Res<RocketPuzzles>,
                     progress: Res<LevelProgress>| {
                        build_level(
                            &mut commands,
                            &assets,
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

            fn playing_state(&self) -> PlayingState {
                self.app
                    .world()
                    .resource::<State<PlayingState>>()
                    .get()
                    .clone()
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
                run.playing_state(),
                PlayingState::MissionComplete,
                "stepping out of the rocket did not finish the run"
            );
        }
    }

    /// Every kind of job has to be done before the run is clear: the panel
    /// *and* each breach.
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
    }
}
