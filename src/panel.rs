//! The isolation panel: three switches and the LEDs over them, bolted to the
//! wall of one room of the rocket.
//!
//! Which room it is in, and which way the three switches have to be thrown, are
//! picked fresh at the start of every run — the point of it is that a player who
//! has run the rocket before still has to find the panel and work it rather than
//! walking a route from memory.
//!
//! The LEDs are wired together rather than one to a switch: they all come on at
//! once, and only when the whole set matches. Lighting each lamp as its own
//! switch came good would let the combination be read off the panel itself,
//! which is not where it is meant to be read from — the setting is printed in
//! the repair manual, on the page the panel has to itself. Finding the room and
//! reading the page is the job; the switches are just where it is signed off.
//!
//! Working the panel is the same press as working a door, and the panel is
//! mounted far enough along the room that the two are never both in reach — see
//! [`Room::fixture`] and the test that holds it there.

use bevy::prelude::*;

use crate::config::{PLAYER_HEIGHT, PLAYER_WIDTH};
use crate::level::{Level, LevelProgress, Room};
use crate::player::Player;
use crate::puzzles::{RocketPuzzles, scramble};
use crate::ui::MUTED_TEXT;

/// How many switches a panel has. The combination is one bit per switch, so
/// this is also what says how many settings there are to work through.
pub const SWITCH_COUNT: usize = 3;

/// A switch and the slot it is thrown in. The slot is what is drawn; the toggle
/// is the smaller block that slides up and down inside it.
const SLOT_SIZE: Vec2 = Vec2::new(28.0, 48.0);
const TOGGLE_SIZE: Vec2 = Vec2::new(24.0, 22.0);
/// How far the toggle sits off the middle of its slot, up when the switch is on
/// and down when it is off.
const TOGGLE_THROW: f32 = 11.0;

const LED_SIZE: Vec2 = Vec2::splat(16.0);

/// Between one switch and the next. Wide enough that the nearest switch to a
/// player standing at the panel is unambiguous, tight enough that all three are
/// worked without walking.
const SWITCH_SPACING: f32 = 56.0;

/// Heights above the floor of the room: the switches at hand height, the lamps
/// above them where they are read at a glance from across the room.
const SWITCH_HEIGHT: f32 = 70.0;
const LED_HEIGHT: f32 = 124.0;

/// Margin the backplate leaves around the fittings on it.
const PLATE_PADDING: f32 = 12.0;

/// Behind the player and the crates so nothing is hidden by it, and in front of
/// the hull tiles it is bolted to.
const PLATE_Z: f32 = -1.6;
const FITTING_Z: f32 = -1.5;

const PLATE_COLOR: Color = Color::srgb(0.20, 0.22, 0.28);
/// The plate goes warm once the panel is set, so a room worked earlier in the
/// run still says so from the doorway.
const PLATE_SOLVED_COLOR: Color = Color::srgb(0.22, 0.34, 0.26);
const SLOT_COLOR: Color = Color::srgb(0.10, 0.11, 0.14);
const TOGGLE_OFF_COLOR: Color = Color::srgb(0.45, 0.47, 0.52);
const TOGGLE_ON_COLOR: Color = Color::srgb(0.85, 0.87, 0.92);
const LED_DARK_COLOR: Color = Color::srgb(0.18, 0.09, 0.09);
const LED_LIT_COLOR: Color = Color::srgb(0.35, 0.95, 0.45);

/// The same margin the doors allow, and for the same reason: the check should
/// never land exactly on the position a player pressed up against something
/// comes to rest in.
const REACH_MARGIN: f32 = 8.0;

/// How far off a switch still counts as standing at it. Half a switch spacing
/// either side, so the whole panel is covered and the nearest switch is the one
/// the player is in front of.
const REACH: Vec2 = Vec2::new(
    SWITCH_SPACING / 2.0 + PLAYER_WIDTH / 2.0 + REACH_MARGIN,
    (SLOT_SIZE.y + PLAYER_HEIGHT) / 2.0,
);

/// The panel of one run: where it is, what it wants, and whether it has been
/// given it. A resource rather than a component because the answer is dealt
/// before the level is built and outlives the geometry built from it.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Panel {
    pub room: Room,
    pub combination: [bool; SWITCH_COUNT],
    /// Latched: once the switches have been matched the panel stays solved, the
    /// same way an opened door stays open. Charging a player the same work twice
    /// is not what the clock is there for.
    pub solved: bool,
}

impl Default for Panel {
    fn default() -> Self {
        Self::from_seed(0)
    }
}

impl Panel {
    /// The room and the combination for one run.
    ///
    /// The room comes from the same deal that places the breaches, off the same
    /// seed, so the panel is never installed in a room a portal is already
    /// standing in.
    ///
    /// The combination is drawn from the settings with at least one switch up,
    /// never from all eight: the switches are spawned down, and a combination of
    /// all-down would be a panel that was solved before the player had found the
    /// room it is in.
    pub fn from_seed(seed: u64) -> Self {
        let bits = scramble(seed);
        let room = RocketPuzzles::from_seed(seed).panel_room;

        let settings = (1u64 << SWITCH_COUNT) - 1;
        let pattern = 1 + scramble(bits) % settings;
        let combination = std::array::from_fn(|index| pattern & (1 << index) != 0);

        Self {
            room,
            combination,
            solved: false,
        }
    }

    fn matched(&self, switches: [bool; SWITCH_COUNT]) -> bool {
        switches == self.combination
    }

    /// The combination as the repair manual prints it: the setting each switch
    /// is to be left in, numbered the way they are mounted, left to right.
    ///
    /// This is the whole reason the manual is worth opening. Without it the
    /// panel is eight settings to be tried in turn against the clock, which is
    /// busywork rather than a job — with it, the manual says what to do and the
    /// run is the doing of it.
    pub fn printed_settings(&self) -> String {
        let settings: Vec<String> = self
            .combination
            .iter()
            .enumerate()
            .map(|(index, up)| format!("{}: {}", index + 1, if *up { "UP" } else { "DOWN" }))
            .collect();

        format!("     {}", settings.join("     "))
    }

    /// The HUD's line on the run's outstanding work.
    ///
    /// The panel says which room it is in and whether it has been set. The
    /// breaches only say how many are left: they are lit, they pulse, and the
    /// crossing goes through every room, so they are met rather than looked for.
    fn status(&self, level: Level, progress: &LevelProgress) -> String {
        let mut line = if level.rooms().contains(&self.room) {
            let state = if self.solved { "SET" } else { "UNSET" };

            format!("Isolation panel · {} · {state}", self.room.label())
        } else {
            String::new()
        };

        if progress.total_portals > 0 {
            if !line.is_empty() {
                line.push_str("     ");
            }
            line.push_str(&format!(
                "Breaches sealed · {}/{}",
                progress.completed_portals.min(progress.total_portals),
                progress.total_portals
            ));
        }

        line
    }
}

/// One of the panel's switches. `index` is its place along the panel, left to
/// right, and what ties it to its bit of the combination.
#[derive(Component, Debug, Clone, Copy)]
pub struct Switch {
    pub index: usize,
    pub on: bool,
    /// The middle of the slot this toggle slides in. Kept here rather than
    /// worked back out of the toggle's own position, which is never the middle
    /// of anything: it is always thrown one way or the other.
    home: f32,
}

/// The lamp over a switch. It carries no state of its own: what it shows is
/// whether the panel as a whole is solved.
#[derive(Component)]
pub struct Led;

/// The plate the fittings are bolted to.
#[derive(Component)]
pub struct Backplate;

/// The HUD line naming the room the panel is in and whether it has been solved.
/// The room is given away on purpose — hunting six rooms for an unmarked panel
/// is not the puzzle, and the clock is short.
#[derive(Component)]
pub struct PanelStatus;

/// Where the `index`th switch is, in world space.
fn switch_at(mount: Vec2, index: usize) -> Vec2 {
    let offset = (index as f32 - (SWITCH_COUNT as f32 - 1.0) / 2.0) * SWITCH_SPACING;

    Vec2::new(mount.x + offset, mount.y + SWITCH_HEIGHT)
}

/// Every position a player has to be able to stand in to work the panel — one
/// per switch, on the floor of its room. Only the layout tests ask for this; the
/// game itself works off where the player already is.
#[cfg(test)]
pub fn working_positions(panel: &Panel) -> [Vec2; SWITCH_COUNT] {
    let mount = panel.room.fixture();

    std::array::from_fn(|index| Vec2::new(switch_at(mount, index).x, mount.y + PLAYER_HEIGHT / 2.0))
}

/// The plate's footprint, so the layout tests can ask what the panel actually
/// covers rather than re-deriving it.
pub fn plate_bounds(panel: &Panel) -> Rect {
    let mount = panel.room.fixture();
    let half_width =
        (SWITCH_COUNT as f32 - 1.0) * SWITCH_SPACING / 2.0 + SLOT_SIZE.x / 2.0 + PLATE_PADDING;

    Rect {
        min: Vec2::new(
            mount.x - half_width,
            mount.y + SWITCH_HEIGHT - SLOT_SIZE.y / 2.0 - PLATE_PADDING,
        ),
        max: Vec2::new(
            mount.x + half_width,
            mount.y + LED_HEIGHT + LED_SIZE.y / 2.0 + PLATE_PADDING,
        ),
    }
}

/// Builds the panel into the level, if this is the level its room is on.
pub fn spawn_panel(
    commands: &mut Commands,
    panel: &Panel,
    level: Level,
    marker: impl Bundle + Clone,
) {
    if !level.rooms().contains(&panel.room) {
        return;
    }

    let mount = panel.room.fixture();
    let plate = plate_bounds(panel);

    commands.spawn((
        marker.clone(),
        Backplate,
        Sprite {
            color: PLATE_COLOR,
            custom_size: Some(plate.size()),
            ..default()
        },
        Transform::from_xyz(plate.center().x, plate.center().y, PLATE_Z),
    ));

    for index in 0..SWITCH_COUNT {
        let at = switch_at(mount, index);

        // The slot the toggle slides in. Static, so it is the one part of a
        // switch that never has to be looked at again once it is up.
        commands.spawn((
            marker.clone(),
            Sprite {
                color: SLOT_COLOR,
                custom_size: Some(SLOT_SIZE),
                ..default()
            },
            Transform::from_xyz(at.x, at.y, FITTING_Z),
        ));

        commands.spawn((
            marker.clone(),
            Switch {
                index,
                on: false,
                home: at.y,
            },
            Sprite {
                color: TOGGLE_OFF_COLOR,
                custom_size: Some(TOGGLE_SIZE),
                ..default()
            },
            Transform::from_xyz(at.x, at.y - TOGGLE_THROW, FITTING_Z + 0.05),
        ));

        commands.spawn((
            marker.clone(),
            Led,
            Sprite {
                color: LED_DARK_COLOR,
                custom_size: Some(LED_SIZE),
                ..default()
            },
            Transform::from_xyz(at.x, mount.y + LED_HEIGHT, FITTING_Z),
        ));
    }
}

/// Throws the nearest switch the player is standing at, and reads the panel
/// afterwards to see whether that was the last one it was waiting for.
///
/// Nearest rather than first, for the same reason the doors do it: the player
/// means the switch they are in front of.
pub fn flip_switches(
    keys: Res<ButtonInput<KeyCode>>,
    mut panel: ResMut<Panel>,
    players: Query<&Transform, With<Player>>,
    mut switches: Query<(Entity, &mut Switch), Without<Player>>,
) {
    // A solved panel is done with. Leaving it live would let a player throw the
    // combination back out again by walking past it.
    if panel.solved || !keys.just_pressed(KeyCode::KeyE) {
        return;
    }

    let Ok(player) = players.single() else {
        return;
    };
    let at = player.translation.truncate();
    let mount = panel.room.fixture();

    let nearest = switches
        .iter()
        .filter(|(_, switch)| in_reach(switch_at(mount, switch.index), at))
        .min_by(|(_, one), (_, other)| {
            let distance = |switch: &Switch| switch_at(mount, switch.index).distance_squared(at);

            distance(one).total_cmp(&distance(other))
        })
        .map(|(entity, _)| entity);

    let Some(entity) = nearest else {
        return;
    };
    if let Ok((_, mut switch)) = switches.get_mut(entity) {
        switch.on = !switch.on;
    }

    let mut thrown = [false; SWITCH_COUNT];
    for (_, switch) in &switches {
        if let Some(state) = thrown.get_mut(switch.index) {
            *state = switch.on;
        }
    }

    if panel.matched(thrown) {
        panel.solved = true;
    }
}

/// Whether a player standing at `player` is at the switch drawn at `switch`.
/// Measured off the middle of the slot rather than the toggle in it, which
/// moves.
fn in_reach(switch: Vec2, player: Vec2) -> bool {
    let offset = (player - switch).abs();

    offset.x <= REACH.x && offset.y <= REACH.y
}

/// Draws what the panel currently is: each toggle thrown its way, and the lamps
/// lit or dark together.
///
/// The filters are what they are because a toggle, a lamp and the plate are all
/// sprites: each query has to say which of the three it means.
#[allow(clippy::type_complexity)]
pub fn light_panel(
    panel: Res<Panel>,
    mut toggles: Query<(&Switch, &mut Sprite, &mut Transform)>,
    mut leds: Query<&mut Sprite, (With<Led>, Without<Switch>)>,
    mut plates: Query<&mut Sprite, (With<Backplate>, Without<Switch>, Without<Led>)>,
) {
    for (switch, mut sprite, mut transform) in &mut toggles {
        sprite.color = if switch.on {
            TOGGLE_ON_COLOR
        } else {
            TOGGLE_OFF_COLOR
        };
        transform.translation.y = switch.home
            + if switch.on {
                TOGGLE_THROW
            } else {
                -TOGGLE_THROW
            };
    }

    let lamp = if panel.solved {
        LED_LIT_COLOR
    } else {
        LED_DARK_COLOR
    };
    for mut sprite in &mut leds {
        sprite.color = lamp;
    }

    for mut sprite in &mut plates {
        sprite.color = if panel.solved {
            PLATE_SOLVED_COLOR
        } else {
            PLATE_COLOR
        };
    }
}

/// Spawned as part of the HUD, so it lives and dies with the rest of the run.
pub fn spawn_panel_status(
    parent: &mut ChildSpawnerCommands,
    level: Level,
    panel: &Panel,
    progress: &LevelProgress,
    font_size: f32,
) {
    parent.spawn((
        PanelStatus,
        Text::new(panel.status(level, progress)),
        TextFont {
            font_size: FontSize::Px(font_size),
            ..default()
        },
        TextColor(MUTED_TEXT),
    ));
}

pub fn sync_panel_status(
    level: Res<Level>,
    panel: Res<Panel>,
    progress: Res<LevelProgress>,
    mut labels: Query<&mut Text, With<PanelStatus>>,
) {
    if !level.is_changed() && !panel.is_changed() && !progress.is_changed() {
        return;
    }

    for mut text in &mut labels {
        **text = panel.status(*level, &progress);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::level::ROOM_COUNT;

    /// Every room has to be one the panel can turn up in, and every setting bar
    /// all-down one it can ask for — otherwise a run's opening is narrower than
    /// it looks. Walked over enough seeds to see the whole of both.
    #[test]
    fn every_room_and_every_workable_combination_comes_up() {
        let mut rooms = [false; ROOM_COUNT];
        let mut combinations = [false; 1 << SWITCH_COUNT];

        for seed in 0..2_000u64 {
            let panel = Panel::from_seed(seed);

            let room = (0..ROOM_COUNT)
                .position(|index| Room::from_index(index) == panel.room)
                .expect("the panel picked a room that is not in the rocket");
            rooms[room] = true;

            let pattern = panel
                .combination
                .iter()
                .enumerate()
                .fold(0usize, |bits, (index, on)| bits | (*on as usize) << index);
            combinations[pattern] = true;
        }

        assert!(rooms.iter().all(|seen| *seen), "some room is never picked");
        assert!(
            !combinations[0],
            "the panel asked for the setting it is spawned in"
        );
        assert!(
            combinations[1..].iter().all(|seen| *seen),
            "some combination is never asked for"
        );
    }

    /// Two runs a moment apart must not be the same run. The seed is the app's
    /// uptime in nanoseconds, so this is what says nearby seeds diverge.
    #[test]
    fn seeds_a_moment_apart_give_different_panels() {
        let seed = 1_234_567_890_u64;
        let differ = (1..=8).filter(|step| Panel::from_seed(seed + step) != Panel::from_seed(seed));

        assert!(differ.count() >= 6, "the pick barely moves between seeds");
    }

    #[test]
    fn a_panel_starts_unsolved_and_needs_its_own_combination() {
        let panel = Panel::from_seed(7);

        assert!(!panel.solved);
        assert!(!panel.matched([false; SWITCH_COUNT]));
        assert!(panel.matched(panel.combination));
    }

    /// A player standing at a switch has to be able to reach it, and a player at
    /// the far end of the panel must not reach the one at the other end — or the
    /// three switches would be one switch worked three times.
    #[test]
    fn each_switch_is_worked_from_in_front_of_it() {
        let panel = Panel::from_seed(0);
        let mount = panel.room.fixture();
        let standing = working_positions(&panel);

        for (index, at) in standing.iter().enumerate() {
            assert!(
                in_reach(switch_at(mount, index), *at),
                "switch {index} cannot be worked from in front of it"
            );
        }

        assert!(
            !in_reach(switch_at(mount, 0), standing[SWITCH_COUNT - 1]),
            "the far switch is worked from the other end of the panel"
        );
    }

    /// `E` works a door and throws a switch, so the two must never be in reach
    /// at once — a press meant for one of them landing on both would open the
    /// bulkhead every time the player touched the panel.
    #[test]
    fn a_panel_is_never_in_reach_of_a_door() {
        for index in 0..ROOM_COUNT {
            let panel = Panel {
                room: Room::from_index(index),
                ..Panel::from_seed(0)
            };

            for at in working_positions(&panel) {
                for door in Level::Rocket.doors() {
                    assert!(
                        !door.in_reach(at),
                        "the panel in {} is worked from the same spot as {door:?}",
                        panel.room.label()
                    );
                }
            }
        }
    }

    /// The other thing in a room: the panel must not be drawn over a ladder, and
    /// nobody should have to stand in the ladder's hold to work a switch.
    #[test]
    fn a_panel_is_clear_of_the_ladders_and_the_hull() {
        for index in 0..ROOM_COUNT {
            let panel = Panel {
                room: Room::from_index(index),
                ..Panel::from_seed(0)
            };
            let plate = plate_bounds(&panel);

            for ladder in Level::Rocket.ladders() {
                let column = ladder.reach();
                let clear = plate.max.x <= column.min.x || plate.min.x >= column.max.x;

                assert!(
                    clear,
                    "the panel in {} is hung over the ladder at x={}",
                    panel.room.label(),
                    ladder.x
                );
            }

            for wall in Level::Rocket.walls() {
                assert!(
                    plate.max.x < wall.centre.x || plate.min.x > wall.centre.x,
                    "the panel in {} is hung through a wall",
                    panel.room.label()
                );
            }
        }
    }

    /// The panel has to be inside the room it is in — under the deck above it,
    /// and above the one it stands on.
    #[test]
    fn a_panel_hangs_inside_its_own_room() {
        use crate::config::PLATFORM_HEIGHT;

        for index in 0..ROOM_COUNT {
            let room = Room::from_index(index);
            let panel = Panel {
                room,
                ..Panel::from_seed(0)
            };
            let plate = plate_bounds(&panel);
            let deck_above = Room::from_index(index).floor() + DECK_HEIGHT_FOR_TESTS;

            assert!(
                plate.min.y > room.floor(),
                "the panel is sunk into the deck"
            );
            assert!(
                plate.max.y < deck_above - PLATFORM_HEIGHT,
                "the panel is driven through the deck above"
            );
        }
    }

    /// Read off the rooms rather than hard-coded, so this follows the layout if
    /// the decks are ever moved.
    const DECK_HEIGHT_FOR_TESTS: f32 = Room::from_index(2).floor() - Room::from_index(0).floor();

    /// The panel is always somewhere the run goes: the rocket is the whole of
    /// it, and every room of the rocket is on the crossing.
    #[test]
    fn the_panel_is_always_in_a_room_of_the_run() {
        for seed in 0..256u64 {
            let panel = Panel::from_seed(seed);

            assert!(
                Level::Rocket.rooms().contains(&panel.room),
                "seed {seed} put the panel outside the rocket"
            );
        }
    }

    /// The panel as it is actually worked: a real player entity walked from one
    /// switch to the next, pressing `E`, against the same systems the run uses.
    /// The layout tests above say the fittings are in the right places; these say
    /// that throwing them does something.
    mod working_it {
        use super::*;
        use crate::player::Player;

        struct Bench {
            app: App,
            player: Entity,
        }

        impl Bench {
            fn with(panel: Panel) -> Self {
                let mut app = App::new();
                app.add_plugins(MinimalPlugins);
                app.insert_resource(panel);
                app.insert_resource(ButtonInput::<KeyCode>::default());
                app.add_systems(Startup, |mut commands: Commands, panel: Res<Panel>| {
                    spawn_panel(&mut commands, &panel, Level::Rocket, ());
                });
                app.add_systems(Update, (flip_switches, light_panel).chain());
                app.update();

                // Off the panel to begin with, so nothing is in reach until a
                // switch is walked up to.
                let player = app
                    .world_mut()
                    .spawn((Player, Transform::from_xyz(0.0, 0.0, 0.0)))
                    .id();

                Self { app, player }
            }

            fn stand_at(&mut self, at: Vec2) {
                let mut transform = self
                    .app
                    .world_mut()
                    .entity_mut(self.player)
                    .into_mut::<Transform>()
                    .expect("the player lost its transform");

                transform.translation = at.extend(0.0);
            }

            /// One frame with `E` down, then one without, so the next press is a
            /// press again rather than a key that was already held.
            fn press(&mut self) {
                let mut keys = ButtonInput::<KeyCode>::default();
                keys.press(KeyCode::KeyE);
                self.app.world_mut().insert_resource(keys);
                self.app.update();

                self.app
                    .world_mut()
                    .insert_resource(ButtonInput::<KeyCode>::default());
                self.app.update();
            }

            fn throw(&mut self, at: Vec2) {
                self.stand_at(at);
                self.press();
            }

            fn panel(&self) -> Panel {
                *self.app.world().resource::<Panel>()
            }

            fn switches(&mut self) -> [bool; SWITCH_COUNT] {
                let mut thrown = [false; SWITCH_COUNT];
                let mut query = self.app.world_mut().query::<&Switch>();

                for switch in query.iter(self.app.world()) {
                    thrown[switch.index] = switch.on;
                }
                thrown
            }

            fn lamps_lit(&mut self) -> bool {
                let mut query = self.app.world_mut().query_filtered::<&Sprite, With<Led>>();
                let mut lamps = query.iter(self.app.world()).peekable();

                lamps.peek().is_some()
                    && query
                        .iter(self.app.world())
                        .all(|led| led.color == LED_LIT_COLOR)
            }
        }

        #[test]
        fn a_panel_is_built_with_three_switches_and_three_lamps_all_dark() {
            let mut bench = Bench::with(Panel::from_seed(3));

            assert_eq!(bench.switches(), [false; SWITCH_COUNT]);
            assert!(!bench.lamps_lit(), "the lamps are lit on an unsolved panel");
            assert_eq!(
                bench
                    .app
                    .world_mut()
                    .query_filtered::<Entity, With<Led>>()
                    .iter(bench.app.world())
                    .count(),
                SWITCH_COUNT
            );
        }

        #[test]
        fn a_press_at_a_switch_throws_that_switch_and_no_other() {
            let panel = Panel::from_seed(3);
            let mut bench = Bench::with(panel);
            let standing = working_positions(&panel);

            bench.throw(standing[1]);

            assert_eq!(bench.switches(), [false, true, false]);
        }

        #[test]
        fn a_press_away_from_the_panel_throws_nothing() {
            let panel = Panel::from_seed(3);
            let mut bench = Bench::with(panel);

            bench.throw(panel.room.fixture() + Vec2::new(400.0, 0.0));

            assert_eq!(bench.switches(), [false; SWITCH_COUNT]);
        }

        /// The whole of it: work the switches into the combination and the panel
        /// solves and the lamps come on. Run over every seed's worth of room and
        /// combination, so it is not one lucky layout that works.
        #[test]
        fn setting_the_combination_solves_the_room_and_lights_the_lamps() {
            for seed in 0..24u64 {
                let panel = Panel::from_seed(seed);
                let mut bench = Bench::with(panel);
                let standing = working_positions(&panel);

                for (index, wanted) in panel.combination.iter().enumerate() {
                    if *wanted {
                        bench.throw(standing[index]);
                    }
                }

                assert_eq!(bench.switches(), panel.combination);
                assert!(
                    bench.panel().solved,
                    "the panel in {} was set and not solved",
                    panel.room.label()
                );
                assert!(bench.lamps_lit(), "the lamps stayed dark on a solved panel");
            }
        }

        /// Anything short of the combination leaves the room unsolved — including
        /// every switch up, which is the setting a player who simply throws all
        /// three would land in.
        #[test]
        fn a_partial_or_wrong_setting_leaves_the_room_unsolved() {
            let panel = Panel {
                combination: [true, false, true],
                ..Panel::from_seed(0)
            };
            let standing = working_positions(&panel);

            let mut bench = Bench::with(panel);
            bench.throw(standing[0]);
            assert!(!bench.panel().solved, "one switch of three solved it");
            assert!(!bench.lamps_lit());

            let mut bench = Bench::with(panel);
            for at in standing {
                bench.throw(at);
            }
            assert_eq!(bench.switches(), [true; SWITCH_COUNT]);
            assert!(!bench.panel().solved, "every switch up solved it");
            assert!(!bench.lamps_lit(), "the lamps lit on a wrong setting");
        }

        /// Once solved it stays solved: the switches lock, so a player crossing
        /// the room again cannot throw the combination back out.
        #[test]
        fn a_solved_panel_cannot_be_thrown_back_out() {
            let panel = Panel {
                combination: [false, true, false],
                ..Panel::from_seed(0)
            };
            let standing = working_positions(&panel);
            let mut bench = Bench::with(panel);

            bench.throw(standing[1]);
            assert!(bench.panel().solved);

            bench.throw(standing[1]);

            assert_eq!(bench.switches(), [false, true, false]);
            assert!(bench.panel().solved);
            assert!(bench.lamps_lit());
        }
    }
}
