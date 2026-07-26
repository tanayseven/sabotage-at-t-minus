//! The level a run is made of.
//!
//! A run is the rocket: however many rooms the chosen difficulty deals, with a
//! job in three of them, and the airlock back at the drop point. The airlock
//! is the way the player came in and the way back out, and working it is what
//! finishes the run — there is nothing after the rocket, so what was once a
//! doorway onto the next level is now the end of the mission.

use bevy::prelude::*;
use rand::Rng;

use crate::config::{DESIGN_HEIGHT, DESIGN_WIDTH, INTERIOR_ZOOM, PLATFORM_HEIGHT, WALL_THICKNESS};
use crate::door::Door;
use crate::ladder::{LADDER_CLEARANCE, Ladder};
use crate::minigames::{CompletedMinigame, MinigameOutcome};
use crate::platform::Platform;
use crate::portal::TriggeredPortal;
use crate::settings::Settings;
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
/// rather than the layout's. Owned rather than `&'static` slices: how many
/// decks the rocket has is picked per run, so the geometry is built fresh each
/// time rather than living in a `const`.
#[derive(Clone)]
pub struct LevelConfig {
    pub platforms: Vec<Platform>,
    pub walls: Vec<Wall>,
    pub ladders: Vec<Ladder>,
    pub doors: Vec<Door>,
    pub crates: Vec<Vec2>,
}

/// How much of the run's breach objective has been sealed.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LevelProgress {
    pub total_portals: usize,
    pub completed_portals: usize,
}

impl LevelProgress {
    pub fn new(level: Level, deck_count: usize) -> Self {
        Self {
            total_portals: level.portal_count(deck_count),
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
        deck_count: usize,
        panel_room: Room,
        panel_solved: bool,
    ) -> bool {
        let panel_done = if level.has_room(deck_count, panel_room) {
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

const LOWER_LADDER_X: f32 = 400.0;
const UPPER_LADDER_X: f32 = -400.0;
const LADDER_GAP: f32 = LADDER_CLEARANCE;
/// The way back out, and working it ends the run. Set into the hull the player
/// is dropped in beside, on the deck they are dropped onto: the run leaves the
/// rocket by the point it came in at, so clearing it is a there-and-back through
/// the rooms rather than a one-way climb.
const AIRLOCK_X: f32 = HULL_LEFT + WALL_THICKNESS / 2.0;

/// The floor plate of the `deck`th deck up from the ground.
const fn deck_floor(deck: usize) -> f32 {
    GROUND_TOP + deck as f32 * DECK_HEIGHT
}

/// One past the top deck's floor — the underside of the hull's roof, however
/// many decks the rocket was dealt.
const fn rocket_ceiling(deck_count: usize) -> f32 {
    deck_floor(deck_count)
}

/// Where the ladder connecting deck `boundary` to deck `boundary + 1` stands.
/// Alternates side each boundary up, so the walking route zig-zags rather than
/// climbing the same side of the hull twice running.
const fn ladder_x(boundary: usize) -> f32 {
    if boundary.is_multiple_of(2) {
        LOWER_LADDER_X
    } else {
        UPPER_LADDER_X
    }
}

const AIRLOCK: Door = Door::airlock(AIRLOCK_X, deck_floor(0));

const fn bulkhead_door(deck: usize) -> Door {
    Door::bulkhead(BULKHEAD_X, deck_floor(deck))
}

fn plate(from: f32, to: f32, top: f32) -> Platform {
    Platform::with_top((from + to) / 2.0, top, to - from)
}

/// The bottom deck spans the whole hull unsplit — there is no floor below it
/// for a ladder to come up through. Every deck above it is split around
/// whichever ladder pierces its floor from the one below, and the roof over
/// the top deck is unsplit again, since nothing climbs past it.
fn rocket_platforms(deck_count: usize) -> Vec<Platform> {
    let mut platforms = Vec::with_capacity(deck_count * 2);

    platforms.push(plate(HULL_LEFT, HULL_RIGHT, deck_floor(0)));

    for deck in 1..deck_count {
        let x = ladder_x(deck - 1);
        let floor = deck_floor(deck);
        platforms.push(plate(HULL_LEFT, x - LADDER_GAP / 2.0, floor));
        platforms.push(plate(x + LADDER_GAP / 2.0, HULL_RIGHT, floor));
    }

    platforms.push(plate(HULL_LEFT, HULL_RIGHT, rocket_ceiling(deck_count)));

    platforms
}

/// A bulkhead door on every deck, plus the airlock out on the bottom one.
fn rocket_doors(deck_count: usize) -> Vec<Door> {
    let mut doors: Vec<Door> = (0..deck_count).map(bulkhead_door).collect();
    doors.push(AIRLOCK);
    doors
}

/// The outer hull, plus a bulkhead wall segment per deck gapped for that
/// deck's door.
fn rocket_walls(deck_count: usize) -> Vec<Wall> {
    let ceiling = rocket_ceiling(deck_count);
    let mut walls = vec![
        Wall::between(HULL_LEFT, deck_floor(0), ceiling),
        Wall::between(HULL_RIGHT, deck_floor(0), ceiling),
    ];

    for deck in 0..deck_count {
        let door = bulkhead_door(deck);
        let next_floor = if deck + 1 == deck_count {
            ceiling
        } else {
            deck_floor(deck + 1)
        };

        walls.push(Wall::between(
            BULKHEAD_X,
            door.lintel(),
            next_floor - PLATFORM_HEIGHT,
        ));
    }

    walls
}

/// One ladder per gap between consecutive decks.
fn rocket_ladders(deck_count: usize) -> Vec<Ladder> {
    (0..deck_count - 1)
        .map(|boundary| {
            Ladder::new(
                ladder_x(boundary),
                deck_floor(boundary),
                deck_floor(boundary + 1),
            )
        })
        .collect()
}

/// Decoration only, one per deck, alternating side.
fn rocket_crates(deck_count: usize) -> Vec<Vec2> {
    (0..deck_count)
        .map(|deck| {
            let x = if deck.is_multiple_of(2) {
                -300.0
            } else {
                200.0
            };
            Vec2::new(x, deck_floor(deck) + 140.0)
        })
        .collect()
}

const ROCKET_SPAWN: Vec2 = Vec2::new(HULL_LEFT + 120.0, deck_floor(0) + 60.0);

fn rocket_camera(deck_count: usize) -> CameraMode {
    CameraMode::Follow {
        zoom: INTERIOR_ZOOM,
        bounds: Rect::new(
            HULL_LEFT - WALL_THICKNESS,
            deck_floor(0) - PLATFORM_HEIGHT,
            HULL_RIGHT + WALL_THICKNESS,
            rocket_ceiling(deck_count),
        ),
    }
}

fn rocket_config(deck_count: usize) -> LevelConfig {
    LevelConfig {
        platforms: rocket_platforms(deck_count),
        walls: rocket_walls(deck_count),
        ladders: rocket_ladders(deck_count),
        doors: rocket_doors(deck_count),
        crates: rocket_crates(deck_count),
    }
}

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

/// How many rooms the bulkhead cuts each deck into.
pub(crate) const ROOMS_PER_DECK: usize = 2;

/// One of the rocket's rooms: the stretch of a deck on one side of the
/// bulkhead. Described by which deck and which side rather than by its corners,
/// because that is what a room *is* here — the plates, the hull and the
/// bulkhead already say where the walls are, and a second copy of those numbers
/// would only be one to keep in step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Room {
    /// 0 is the deck the player is dropped onto and leaves by, the top one is
    /// the last of the rocket's decks.
    pub deck: usize,
    pub side: Side,
}

/// How far along a room, from the bulkhead toward the hull, a fixture is
/// mounted. Deliberately not the middle: a ladder comes up at 400 units out and
/// a doorway is worked from as far as a crate's width back from the bulkhead,
/// so the middle of the room is the one place a fixture would be in the way of
/// both. `a_panel_is_clear_of_everything_else_in_its_room` holds this.
const FIXTURE_ALONG_ROOM: f32 = 0.45;

/// How far along a room its code board is hung: back toward the bulkhead, so a
/// player coming through the doorway reads it before they are past it, and well
/// short of the panel and the breach further out.
const SIGN_ALONG_ROOM: f32 = 0.2;
/// Above the doorway rather than beside it, where nothing else is mounted and
/// nothing loose can be shoved in front of it.
const SIGN_HEIGHT: f32 = 168.0;

/// How many characters a room code is. Four, so it reads as a plate marking
/// rather than a name.
const ROOM_CODE_LEN: usize = 4;
const ROOM_CODE_ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

/// The code stencilled by each room's hatch, in room order — what the manual's
/// room index lists the rooms under, and so how a player turns the code the HUD
/// gives them into somewhere to walk.
///
/// Drawn fresh at the start of every run rather than written down here, so the
/// codes are not something a player can learn once and then stop opening the
/// manual — and sized to that run's room count, since how many decks the
/// rocket has is picked per run too.
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct RoomCodes(Vec<String>);

impl RoomCodes {
    /// Distinct codes, because the whole use of a code is telling one room
    /// from the rest of them.
    pub fn random(room_count: usize) -> Self {
        let mut rng = rand::thread_rng();
        let mut codes: Vec<String> = Vec::with_capacity(room_count);

        while codes.len() < room_count {
            let code: String = (0..ROOM_CODE_LEN)
                .map(|_| {
                    let pick = rng.gen_range(0..ROOM_CODE_ALPHABET.len());
                    ROOM_CODE_ALPHABET[pick] as char
                })
                .collect();

            // A draw that came out all letters or all digits reads as a word or
            // a number rather than a marking, so it goes back in the hat.
            let mixed = code
                .chars()
                .any(|character| character.is_ascii_alphabetic())
                && code.chars().any(|character| character.is_ascii_digit());

            if mixed && !codes.contains(&code) {
                codes.push(code);
            }
        }

        Self(codes)
    }

    /// The code stencilled by this room's hatch, without its `#`.
    pub fn of(&self, room: Room) -> &str {
        &self.0[room.index()]
    }
}

/// One deck's worth of the manual's room index: both its rooms, code first.
/// Built from the same room list the level is, so the codes in the manual
/// cannot drift from the codes on the rooms.
pub fn deck_index_line(codes: &RoomCodes, deck: usize) -> String {
    let port = Room {
        deck,
        side: Side::Port,
    };
    let starboard = Room {
        deck,
        side: Side::Starboard,
    };

    format!(
        "  #{}  {:<20}#{}  {}",
        codes.of(port),
        port.label(),
        codes.of(starboard),
        starboard.label()
    )
}

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
        deck_floor(self.deck)
    }

    /// Where a wall fixture is mounted in this room, given as the point on the
    /// floor it stands on, so what is hung there decides its own height.
    pub const fn fixture(self) -> Vec2 {
        Vec2::new(
            BULKHEAD_X + (self.side.hull() - BULKHEAD_X) * FIXTURE_ALONG_ROOM,
            self.floor(),
        )
    }

    /// The centre of a breach opened in this room, over the same clear stretch
    /// of wall the panel is bolted to.
    ///
    /// Every room has a breach now, so one room has both — and they are drawn
    /// on top of each other rather than side by side. That is deliberate: the
    /// stretch of a room a player actually walks is short, hemmed in by the
    /// doorway at one end and the ladder at the other, and a breach moved off
    /// it to make space is a breach that can be strolled past. A cleared breach
    /// despawns, so the room that draws both is worked breach first and panel
    /// second, which is the order the lit switches want anyway.
    pub const fn portal_mount(self) -> Vec2 {
        let at = self.fixture();

        Vec2::new(at.x, at.y + PORTAL_MOUNT_HEIGHT)
    }

    /// Where this room's code board hangs.
    pub const fn sign(self) -> Vec2 {
        Vec2::new(
            BULKHEAD_X + (self.side.hull() - BULKHEAD_X) * SIGN_ALONG_ROOM,
            self.floor() + SIGN_HEIGHT,
        )
    }

    /// The inverse of [`Room::from_index`] — which room of the rocket this is.
    pub const fn index(self) -> usize {
        self.deck * ROOMS_PER_DECK
            + match self.side {
                Side::Port => 0,
                Side::Starboard => 1,
            }
    }

    /// How the room is named in anything the player reads.
    pub fn label(self) -> String {
        format!("deck {}, {}", self.deck, self.side.name())
    }
}

fn rocket_rooms(deck_count: usize) -> Vec<Room> {
    (0..deck_count * ROOMS_PER_DECK)
        .map(Room::from_index)
        .collect()
}

impl Level {
    /// How the level is named to the player, in the same terms the launch pad
    /// names itself: where you are, not what number it is.
    pub fn title(self) -> &'static str {
        match self {
            Level::Rocket => "Inside Rocket",
        }
    }

    pub fn config(self, deck_count: usize) -> LevelConfig {
        match self {
            Level::Rocket => rocket_config(deck_count),
        }
    }

    pub fn platforms(self, deck_count: usize) -> Vec<Platform> {
        self.config(deck_count).platforms
    }

    pub fn walls(self, deck_count: usize) -> Vec<Wall> {
        self.config(deck_count).walls
    }

    pub fn ladders(self, deck_count: usize) -> Vec<Ladder> {
        self.config(deck_count).ladders
    }

    pub fn doors(self, deck_count: usize) -> Vec<Door> {
        self.config(deck_count).doors
    }

    /// The rooms the level is divided into.
    pub fn rooms(self, deck_count: usize) -> Vec<Room> {
        match self {
            Level::Rocket => rocket_rooms(deck_count),
        }
    }

    /// Whether `room` is one of this level's, at this run's deck count.
    /// Cheaper than `self.rooms(deck_count).contains(&room)` for the systems
    /// that ask on every frame a value changes — no room list to build just to
    /// throw away.
    pub fn has_room(self, deck_count: usize, room: Room) -> bool {
        match self {
            Level::Rocket => room.deck < deck_count,
        }
    }

    pub fn crates(self, deck_count: usize) -> Vec<Vec2> {
        self.config(deck_count).crates
    }

    pub fn player_spawn(self) -> Vec2 {
        match self {
            Level::Rocket => ROCKET_SPAWN,
        }
    }

    /// How many breaches this level puts up, which is what the objective counts
    /// down: one per kind of challenge, one room each, however the rooms happen
    /// to be dealt.
    /// One breach per room, so there is something to work wherever the player
    /// goes and the airlock waits on all of them.
    pub fn portal_count(self, deck_count: usize) -> usize {
        match self {
            Level::Rocket => deck_count * ROOMS_PER_DECK,
        }
    }

    pub fn camera(self, deck_count: usize) -> CameraMode {
        match self {
            Level::Rocket => rocket_camera(deck_count),
        }
    }

    /// The stretch of level the hull lining is papered over. Taken from what the
    /// camera can reach rather than from the hull itself, so a pan that runs to
    /// the edge of the level shows plating rather than the void behind it.
    pub fn interior(self, deck_count: usize) -> Rect {
        match self.camera(deck_count) {
            CameraMode::Follow { bounds, .. } => bounds,
            CameraMode::Fixed => {
                Rect::from_center_size(Vec2::ZERO, Vec2::new(DESIGN_WIDTH, DESIGN_HEIGHT))
            }
        }
    }
}

/// Inserted rather than assigned, so change detection fires — and the camera
/// re-frames — every time a run starts. The room codes are drawn here too,
/// sized to the difficulty just chosen, rather than once at boot: the deck
/// count varies run to run, and a stale set of codes would not even have one
/// for every room.
pub fn reset_level(mut commands: Commands, settings: Res<Settings>) {
    let room_count = settings.difficulty.deck_count() * ROOMS_PER_DECK;

    commands.insert_resource(Level::default());
    commands.insert_resource(RoomCodes::random(room_count));
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
        AIRLOCK, BULKHEAD_X, CameraMode, DECK_HEIGHT, Door, HULL_LEFT, HULL_RIGHT, LADDER_GAP,
        Level, LevelProgress, ROOM_CODE_LEN, ROOMS_PER_DECK, Room, RoomCodes, deck_floor, ladder_x,
    };
    use crate::config::PLAYER_HEIGHT;

    /// The whole test module works against a fixed deck count, standing in
    /// for whichever difficulty a real run picks.
    const TEST_DECK_COUNT: usize = 4;
    const TEST_ROOM_COUNT: usize = TEST_DECK_COUNT * ROOMS_PER_DECK;

    /// The codes are the only thing telling one room from another in the HUD,
    /// and the manual is only worth opening if they read as markings.
    #[test]
    fn room_codes_are_distinct_markings() {
        for _ in 0..32 {
            let codes = RoomCodes::random(TEST_ROOM_COUNT);
            let drawn: Vec<&str> = (0..TEST_ROOM_COUNT)
                .map(|index| codes.of(Room::from_index(index)))
                .collect();

            for (index, code) in drawn.iter().enumerate() {
                assert_eq!(
                    code.chars().count(),
                    ROOM_CODE_LEN,
                    "{code:?} is the wrong length"
                );
                assert!(
                    code.chars()
                        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()),
                    "{code:?} is not alphanumeric"
                );
                assert!(
                    code.chars().any(|c| c.is_ascii_alphabetic())
                        && code.chars().any(|c| c.is_ascii_digit()),
                    "{code:?} reads as a word or a number rather than a marking"
                );
                assert!(
                    !drawn[index + 1..].contains(code),
                    "two rooms were stencilled {code:?}"
                );
            }
        }
    }

    /// The board has to be inside its own room, clear of the panel and the
    /// breach mounted further out.
    #[test]
    fn every_room_signs_itself_inside_its_own_walls() {
        for index in 0..TEST_ROOM_COUNT {
            let room = Room::from_index(index);
            let board = room.sign();

            assert!(
                board.x.abs() > BULKHEAD_X.abs() && board.x.abs() < HULL_RIGHT,
                "the board in {} is outside the hull",
                room.label()
            );
            assert!(
                board.y > room.floor() && board.y < room.floor() + DECK_HEIGHT,
                "the board in {} is not on its own deck",
                room.label()
            );
            assert!(
                (board.x - room.fixture().x).abs() > 40.0,
                "the board in {} is hung on top of the panel",
                room.label()
            );
        }
    }

    #[test]
    fn a_run_opens_inside_the_rocket() {
        assert_eq!(Level::default(), Level::Rocket);
    }

    /// The rocket is the run: there is nothing after it, and the airlock is the
    /// way back out rather than a doorway onto somewhere else.
    #[test]
    fn the_rocket_is_the_whole_run() {
        assert!(!Level::Rocket.walls(TEST_DECK_COUNT).is_empty());
        assert!(!Level::Rocket.ladders(TEST_DECK_COUNT).is_empty());
        // A bulkhead door per deck, plus the airlock out.
        assert_eq!(
            Level::Rocket.doors(TEST_DECK_COUNT).len(),
            TEST_DECK_COUNT + 1
        );
        assert_eq!(AIRLOCK.kind, crate::door::DoorKind::Airlock);
    }

    /// The exit is the entrance: the hatch the player is dropped in beside is
    /// the one they leave by, so the run ends where it began and the crossing is
    /// a round trip. Read off the level's own doors, so a layout that stopped
    /// shipping the airlock fails here rather than passing against the constant.
    #[test]
    fn the_exit_is_where_the_run_starts() {
        let spawn = Level::Rocket.player_spawn();
        let exit = Level::Rocket
            .doors(TEST_DECK_COUNT)
            .into_iter()
            .find(|door| door.kind == crate::door::DoorKind::Airlock)
            .expect("the rocket has no way back out");

        assert!(
            exit.at.x < BULKHEAD_X && spawn.x < BULKHEAD_X,
            "the exit is not on the side of the bulkhead the run starts on"
        );
        assert_eq!(
            exit.sill(),
            deck_floor(0),
            "the exit is not on the deck the run starts on"
        );
        assert!(
            exit.in_reach(spawn),
            "the run does not start within reach of the way out"
        );
    }

    /// A breach in every room, so wherever the player goes there is something
    /// to work — and the airlock, which waits on all of them, cannot be reached
    /// by walking round the jobs.
    #[test]
    fn the_rocket_carries_a_breach_in_every_room() {
        assert_eq!(Level::Rocket.portal_count(TEST_DECK_COUNT), TEST_ROOM_COUNT);
        assert_eq!(Level::Rocket.rooms(TEST_DECK_COUNT).len(), TEST_ROOM_COUNT);
    }

    /// The geometry is built fresh from whichever deck count a difficulty
    /// deals, rather than off a fixed constant — so this is what holds every
    /// tier, from Very Easy's two decks to Very Hard's six, to the shapes the
    /// rest of the module only checks at one fixed count.
    #[test]
    fn the_rocket_s_geometry_holds_at_every_difficulty() {
        use crate::difficulty::Difficulty;

        for difficulty in Difficulty::ALL {
            let deck_count = difficulty.deck_count();
            let room_count = deck_count * ROOMS_PER_DECK;
            let label = difficulty.label();

            let platforms = Level::Rocket.platforms(deck_count);
            let walls = Level::Rocket.walls(deck_count);
            let ladders = Level::Rocket.ladders(deck_count);
            let doors = Level::Rocket.doors(deck_count);
            let rooms = Level::Rocket.rooms(deck_count);

            // One ladder per gap between decks, a bulkhead door per deck plus
            // the airlock, two plates per deck, and the outer hull plus one
            // bulkhead wall segment per deck.
            assert_eq!(ladders.len(), deck_count - 1, "{label}: wrong ladder count");
            assert_eq!(doors.len(), deck_count + 1, "{label}: wrong door count");
            assert_eq!(
                platforms.len(),
                deck_count * 2,
                "{label}: wrong platform count"
            );
            assert_eq!(walls.len(), deck_count + 2, "{label}: wrong wall count");
            assert_eq!(rooms.len(), room_count, "{label}: wrong room count");
            assert_eq!(
                Level::Rocket.portal_count(deck_count),
                room_count,
                "{label}: portal count does not match the room count"
            );

            // Every ladder actually reaches the deck above the one it starts
            // on, and every bulkhead door stands on a deck the rocket has.
            for ladder in &ladders {
                assert!(
                    ladder.head - ladder.foot > 0.0,
                    "{label}: a ladder does not climb"
                );
            }
            for door in &doors {
                assert!(
                    (0..deck_count).any(|deck| door.sill() == deck_floor(deck)),
                    "{label}: a door stands on no deck of the rocket"
                );
            }

            // The camera has to open on the drop point, whatever the hull
            // ended up tall enough for.
            let CameraMode::Follow { bounds, .. } = Level::Rocket.camera(deck_count) else {
                panic!("{label}: the rocket is meant to use a following camera");
            };
            assert!(
                bounds.contains(Level::Rocket.player_spawn()),
                "{label}: the drop point is outside the camera bounds"
            );

            // Distinct codes for every room this difficulty actually has.
            let codes = RoomCodes::random(room_count);
            let drawn: Vec<&str> = (0..room_count)
                .map(|index| codes.of(Room::from_index(index)))
                .collect();
            for (index, code) in drawn.iter().enumerate() {
                assert!(
                    !drawn[index + 1..].contains(code),
                    "{label}: two rooms were stencilled {code:?}"
                );
            }
        }
    }

    /// A breach is walked into rather than worked, so one hung out of the
    /// player's way would be strolled past — and the airlock waits on it, so
    /// that is a run that cannot be finished.
    #[test]
    fn a_portal_in_a_room_is_walked_into() {
        use crate::config::PLAYER_HEIGHT;
        use crate::portal::PORTAL_RADIUS;

        for index in 0..TEST_ROOM_COUNT {
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

        let ladders = Level::Rocket.ladders(TEST_DECK_COUNT);
        let doors = Level::Rocket.doors(TEST_DECK_COUNT);

        for index in 0..TEST_ROOM_COUNT {
            let room = Room::from_index(index);
            let breach = room.portal_mount();

            for ladder in &ladders {
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

            for door in &doors {
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
        let CameraMode::Follow { bounds, .. } = Level::Rocket.camera(TEST_DECK_COUNT) else {
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
        use crate::settings::Settings;
        use crate::setup::{RunState, build_level};
        use crate::state::{GameState, PlayingState};
        use bevy::asset::AssetPlugin;
        use bevy_rapier2d::prelude::*;
        use std::time::Duration;

        const STEP: f32 = 1.0 / 60.0;
        const A: KeyCode = KeyCode::KeyA;
        const D: KeyCode = KeyCode::KeyD;
        const E: KeyCode = KeyCode::KeyE;
        const S: KeyCode = KeyCode::KeyS;
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
                app.init_resource::<Settings>();
                app.add_systems(Startup, configure_physics);
                // The panel is built along with the rest of it, and pinned to a
                // seed so the crossing is run against a rocket with one in it
                // rather than one where the pick happened to go elsewhere.
                app.init_resource::<Panel>();
                app.insert_resource(RocketPuzzles::from_seed(
                    0,
                    TEST_DECK_COUNT * ROOMS_PER_DECK,
                ));
                app.insert_resource(LevelProgress::new(Level::Rocket, TEST_DECK_COUNT));
                app.insert_resource(RoomCodes::random(TEST_ROOM_COUNT));
                app.add_systems(
                    Startup,
                    |mut commands: Commands,
                     assets: Res<AssetServer>,
                     panel: Res<Panel>,
                     puzzles: Res<RocketPuzzles>,
                     codes: Res<RoomCodes>,
                     progress: Res<LevelProgress>| {
                        build_level(
                            &mut commands,
                            &assets,
                            Level::Rocket,
                            TEST_DECK_COUNT,
                            &codes,
                            RunState {
                                puzzles: puzzles.clone(),
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
            assert_shut_out(at, D, deck_floor(0), "deck 0");

            // Work it, carry on, and take the ladder up.
            run.hold(&[E], 2);
            assert!(
                run.hold_until(&[D], 600, |at| at.x >= ladder_x(0)),
                "deck 0's door never let the player through to the lower ladder (stuck at {:?})",
                run.at()
            );
            assert!(
                run.hold_until(&[W], 400, |at| at.y >= standing_on(deck_floor(1)) - 1.0),
                "the lower ladder never reached deck 1"
            );

            // Off the ladder onto solid plate, then back across to deck 1's door.
            assert!(
                run.hold_until(&[A], 300, |at| at.x <= ladder_x(0) - LADDER_GAP),
                "never stepped off the lower ladder onto deck 1"
            );
            let at = run.shut_out_by_the_door(A);
            assert_shut_out(at, A, deck_floor(1), "deck 1");

            // Through it and up the second ladder.
            run.hold(&[E], 2);
            assert!(
                run.hold_until(&[A], 600, |at| at.x <= ladder_x(1)),
                "deck 1's door never let the player through to the upper ladder"
            );
            assert!(
                run.hold_until(&[W], 400, |at| at.y >= standing_on(deck_floor(2)) - 1.0),
                "the upper ladder never reached deck 2"
            );

            // Off it, across deck 2, and through its bulkhead to the third
            // ladder.
            assert!(
                run.hold_until(&[D], 300, |at| at.x >= ladder_x(1) + LADDER_GAP),
                "never stepped off the upper ladder onto deck 2"
            );
            let at = run.shut_out_by_the_door(D);
            assert_shut_out(at, D, deck_floor(2), "deck 2");

            run.hold(&[E], 2);
            assert!(
                run.hold_until(&[D], 600, |at| at.x >= ladder_x(2)),
                "deck 2's door never let the player through to the third ladder (stuck at {:?})",
                run.at()
            );
            assert!(
                run.hold_until(&[W], 400, |at| at.y >= standing_on(deck_floor(3)) - 1.0),
                "the third ladder never reached deck 3"
            );

            // Off it, across the top deck, and through the last bulkhead.
            assert!(
                run.hold_until(&[A], 300, |at| at.x <= ladder_x(2) - LADDER_GAP),
                "never stepped off the third ladder onto deck 3"
            );
            let at = run.shut_out_by_the_door(A);
            assert_shut_out(at, A, deck_floor(3), "deck 3");

            run.hold(&[E], 2);
            assert!(
                run.hold_until(&[A], 600, |at| at.x <= HULL_LEFT + 200.0),
                "deck 3's door never let the player through to the port end (stuck at {:?})",
                run.at()
            );

            // The way out is back where the run started, so the top deck is not
            // the end of it: back down all three ladders and along deck 0 to
            // the airlock the player was dropped in beside.
            assert!(
                run.hold_until(&[D], 600, |at| at.x >= ladder_x(2)),
                "never got back across deck 3 to the third ladder"
            );
            assert!(
                run.hold_until(&[S], 400, |at| at.y <= standing_on(deck_floor(2)) + 1.0),
                "the third ladder never brought the player back down to deck 2"
            );
            assert!(
                run.hold_until(&[A], 600, |at| at.x <= ladder_x(1)),
                "never got back across deck 2 to the upper ladder"
            );
            assert!(
                run.hold_until(&[S], 400, |at| at.y <= standing_on(deck_floor(1)) + 1.0),
                "the upper ladder never brought the player back down to deck 1"
            );
            assert!(
                run.hold_until(&[D], 600, |at| at.x >= ladder_x(0)),
                "never crossed deck 1 back to the lower ladder"
            );
            assert!(
                run.hold_until(&[S], 400, |at| at.y <= standing_on(deck_floor(0)) + 1.0),
                "the lower ladder never brought the player back down to deck 0"
            );
            assert!(
                run.hold_until(&[A], 800, |at| AIRLOCK.in_reach(at)),
                "deck 0 never led back to the airlock (stuck at {:?})",
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
                Level::Rocket.portal_count(TEST_DECK_COUNT),
                "the crossing walked past a breach without triggering it"
            );

            // The final airlock only opens after every job on the level is done.
            run.sign_off_the_repairs();
            run.step(&[]);

            // And out. Working the airlock while standing in it is what ends the
            // level, so the walk and the press go in together.
            run.hold(&[E, A], 6);

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
        let rocket_progress = LevelProgress::new(Level::Rocket, TEST_DECK_COUNT);
        let rocket_room = Level::Rocket.rooms(TEST_DECK_COUNT)[0];

        assert!(!rocket_progress.all_obstacles_completed(
            Level::Rocket,
            TEST_DECK_COUNT,
            rocket_room,
            false
        ));
        assert!(
            !rocket_progress.all_obstacles_completed(
                Level::Rocket,
                TEST_DECK_COUNT,
                rocket_room,
                true
            ),
            "the panel alone opened the airlock with breaches still outstanding"
        );

        let mut rocket_done = LevelProgress::new(Level::Rocket, TEST_DECK_COUNT);
        rocket_done.completed_portals = rocket_done.total_portals;
        assert!(!rocket_done.all_obstacles_completed(
            Level::Rocket,
            TEST_DECK_COUNT,
            rocket_room,
            false
        ));
        assert!(rocket_done.all_obstacles_completed(
            Level::Rocket,
            TEST_DECK_COUNT,
            rocket_room,
            true
        ));
    }
}
