//! The isolation panels: a row of switches and the LEDs over them, bolted to
//! the wall of every room of the rocket, one panel to a room.
//!
//! Every panel's combination is picked fresh at the start of every run — the
//! point of it is that a player who has run the rocket before still has to
//! find each panel and work it rather than walking a route from memory.
//!
//! Rooms further into the rocket carry more switches: the panel a player meets
//! on the bottom deck is the easiest of the run, and the one on the top deck
//! the hardest, so the job gets harder the further the crossing goes.
//!
//! The LEDs of a panel are wired together rather than one to a switch: they
//! all come on at once, and only when the whole set matches. Lighting each
//! lamp as its own switch came good would let the combination be read off the
//! panel itself, which is not where it is meant to be read from — the setting
//! is printed in the repair manual. Finding the room and reading the page is
//! the job; the switches are just where it is signed off.
//!
//! Working a panel is the same press as working a door, and a panel is
//! mounted far enough along its room that the two are never both in reach —
//! see [`Room::panel_mount`] and the test that holds it there. Its room's
//! breach is mounted on a different, nearer stretch of wall — see
//! [`Room::portal_mount`] — so a room that carries both can be worked without
//! one hiding the other.

use bevy::prelude::*;

use crate::config::{PLAYER_HEIGHT, PLAYER_WIDTH};
use crate::level::{Level, LevelProgress, Room};
use crate::player::Player;
use crate::puzzles::scramble;
use crate::ui::MUTED_TEXT;

/// How many switches the panel in `room` has. The combination is one bit per
/// switch, so this is also what says how many settings there are to work
/// through — and it climbs a switch a deck, so the panel a run meets last is
/// the one with the most settings to read off the manual and set right.
///
/// Capped rather than left to climb forever: past four switches the panel
/// would no longer fit the clear stretch of wall between the ladder and the
/// hull that every room leaves for it — see [`switch_spacing`].
pub fn switch_count(room: Room) -> usize {
    (3 + room.deck).min(4)
}

/// How far apart a panel of `count` switches spaces them. Held to a fixed
/// footprint regardless of `count` — every room only leaves so much clear
/// wall for it — so a panel with more switches on it packs them closer rather
/// than growing wider than the last one.
fn switch_spacing(count: usize) -> f32 {
    let budget = (PANEL_HALF_WIDTH_BUDGET - SLOT_SIZE.x / 2.0 - PLATE_PADDING) * 2.0;

    (budget / (count as f32 - 1.0).max(1.0)).min(SWITCH_SPACING_AT_THREE)
}

/// A switch and the slot it is thrown in. The slot is what is drawn; the toggle
/// is the smaller block that slides up and down inside it.
const SLOT_SIZE: Vec2 = Vec2::new(28.0, 48.0);
const TOGGLE_SIZE: Vec2 = Vec2::new(24.0, 22.0);
/// How far the toggle sits off the middle of its slot, up when the switch is on
/// and down when it is off.
const TOGGLE_THROW: f32 = 11.0;

const LED_SIZE: Vec2 = Vec2::splat(16.0);

/// The widest a panel's switches are ever spaced, however few of them there
/// are. A three-switch panel would otherwise be spaced no differently from
/// how much room it has, and end up looking sparser the harder a room's own
/// difficulty pushed the budget below what three switches actually need.
const SWITCH_SPACING_AT_THREE: f32 = 56.0;

/// The most a panel's plate is allowed to reach out from its mount, either
/// way, and still clear both the ladder on its near side and the hull on its
/// far side. See `a_panel_is_clear_of_the_ladders_and_the_hull`.
const PANEL_HALF_WIDTH_BUDGET: f32 = 62.0;

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

/// How far off a switch still counts as standing at it, given the spacing its
/// panel was built with. Half a switch spacing either side, so the whole
/// panel is covered and the nearest switch is the one the player is in front
/// of.
fn reach(spacing: f32) -> Vec2 {
    Vec2::new(
        spacing / 2.0 + PLAYER_WIDTH / 2.0 + REACH_MARGIN,
        (SLOT_SIZE.y + PLAYER_HEIGHT) / 2.0,
    )
}

/// One room's panel: what it wants, and whether it has been given it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Panel {
    pub room: Room,
    pub combination: Vec<bool>,
    /// Latched: once the switches have been matched the panel stays solved, the
    /// same way an opened door stays open. Charging a player the same work twice
    /// is not what the clock is there for.
    pub solved: bool,
}

impl Panel {
    /// One room's panel, drawn off `seed` mixed with the room it is in so
    /// that every room's panel is independent of every other's.
    ///
    /// The combination is drawn from the settings with at least one switch up,
    /// never from all-down: the switches are spawned down, and a combination of
    /// all-down would be a panel that was solved before the player had found the
    /// room it is in.
    fn for_room(room: Room, seed: u64) -> Self {
        let count = switch_count(room);
        let bits = scramble(seed ^ scramble(room.index() as u64));

        let settings = (1u64 << count) - 1;
        let pattern = 1 + bits % settings;
        let combination = (0..count)
            .map(|index| pattern & (1 << index) != 0)
            .collect();

        Self {
            room,
            combination,
            solved: false,
        }
    }

    fn matched(&self, switches: &[bool]) -> bool {
        switches == self.combination
    }

    /// The combination as the repair manual prints it: one letter a switch,
    /// `U` for up and `D` for down, left to right as they are mounted.
    pub fn printed_settings(&self) -> String {
        self.combination
            .iter()
            .map(|up| if *up { 'U' } else { 'D' })
            .collect()
    }
}

/// Every room's panel for one run. A resource rather than a component because
/// the answers are dealt before the level is built and outlive the geometry
/// built from them.
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct Panels(Vec<Panel>);

impl Default for Panels {
    /// A placeholder dealt over Medium's room count — every run overwrites
    /// this before it is ever read, the same way
    /// [`crate::puzzles::RocketPuzzles::default`] does.
    fn default() -> Self {
        Self::from_seed(0, Level::Rocket, 4)
    }
}

impl Panels {
    /// One panel a room, for every room `level` has at `deck_count`.
    pub fn from_seed(seed: u64, level: Level, deck_count: usize) -> Self {
        Self(
            level
                .rooms(deck_count)
                .iter()
                .map(|room| Panel::for_room(*room, seed))
                .collect(),
        )
    }

    pub fn of(&self, room: Room) -> &Panel {
        self.0
            .iter()
            .find(|panel| panel.room == room)
            .expect("every room in the level has a panel")
    }

    fn of_mut(&mut self, room: Room) -> &mut Panel {
        self.0
            .iter_mut()
            .find(|panel| panel.room == room)
            .expect("every room in the level has a panel")
    }

    pub fn iter(&self) -> impl Iterator<Item = &Panel> {
        self.0.iter()
    }

    /// Whether every panel in play has been set. The airlock waits on this the
    /// same way it waits on every breach.
    pub fn all_solved(&self) -> bool {
        self.0.iter().all(|panel| panel.solved)
    }

    /// Signs off every panel at once. Only ever used to script a run's ending
    /// in a test — the game itself solves one panel at a time, at its
    /// switches.
    #[cfg(test)]
    pub fn solve_all(&mut self) {
        for panel in &mut self.0 {
            panel.solved = true;
        }
    }

    fn solved_count(&self) -> usize {
        self.0.iter().filter(|panel| panel.solved).count()
    }

    /// The HUD's line on the run's outstanding work.
    fn status(&self, progress: &LevelProgress) -> String {
        let mut line = if !self.0.is_empty() {
            format!("Panels set · {}/{}", self.solved_count(), self.0.len())
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

/// One of a panel's switches. `room` is which panel it belongs to, and
/// `index` is its place along that panel, left to right — what ties it to its
/// bit of the panel's combination.
#[derive(Component, Debug, Clone, Copy)]
pub struct Switch {
    pub room: Room,
    pub index: usize,
    pub on: bool,
    /// The middle of the slot this toggle slides in. Kept here rather than
    /// worked back out of the toggle's own position, which is never the middle
    /// of anything: it is always thrown one way or the other.
    home: f32,
}

/// The lamp over a switch. It carries no state of its own beyond which room's
/// panel it belongs to: what it shows is whether that panel as a whole is
/// solved.
#[derive(Component)]
pub struct Led {
    pub room: Room,
}

/// The plate the fittings are bolted to.
#[derive(Component)]
pub struct Backplate {
    pub room: Room,
}

/// The HUD line naming how many panels have been set.
#[derive(Component)]
pub struct PanelStatus;

/// Where the `index`th switch of `room`'s panel, which has `count` switches,
/// is in world space.
fn switch_at(room: Room, count: usize, index: usize) -> Vec2 {
    let mount = room.panel_mount();
    let spacing = switch_spacing(count);
    let offset = (index as f32 - (count as f32 - 1.0) / 2.0) * spacing;

    Vec2::new(mount.x + offset, mount.y + SWITCH_HEIGHT)
}

/// Every position a player has to be able to stand in to work a panel — one
/// per switch, on the floor of its room. Only the layout tests ask for this; the
/// game itself works off where the player already is.
#[cfg(test)]
pub fn working_positions(panel: &Panel) -> Vec<Vec2> {
    let mount = panel.room.panel_mount();
    let count = panel.combination.len();

    (0..count)
        .map(|index| {
            Vec2::new(
                switch_at(panel.room, count, index).x,
                mount.y + PLAYER_HEIGHT / 2.0,
            )
        })
        .collect()
}

/// A panel's plate footprint, so the layout tests can ask what it actually
/// covers rather than re-deriving it.
pub fn plate_bounds(panel: &Panel) -> Rect {
    let mount = panel.room.panel_mount();
    let count = panel.combination.len();
    let half_width =
        (count as f32 - 1.0) * switch_spacing(count) / 2.0 + SLOT_SIZE.x / 2.0 + PLATE_PADDING;

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

/// Builds every one of `panels`' panels into the level, one to a room.
pub fn spawn_panels(
    commands: &mut Commands,
    panels: &Panels,
    level: Level,
    deck_count: usize,
    marker: impl Bundle + Clone,
) {
    for panel in panels.iter() {
        spawn_panel(commands, panel, level, deck_count, marker.clone());
    }
}

/// Builds one panel into the level, if this is the level its room is on.
fn spawn_panel(
    commands: &mut Commands,
    panel: &Panel,
    level: Level,
    deck_count: usize,
    marker: impl Bundle + Clone,
) {
    if !level.has_room(deck_count, panel.room) {
        return;
    }

    let mount = panel.room.panel_mount();
    let count = panel.combination.len();
    let plate = plate_bounds(panel);

    commands.spawn((
        marker.clone(),
        Backplate { room: panel.room },
        Sprite {
            color: PLATE_COLOR,
            custom_size: Some(plate.size()),
            ..default()
        },
        Transform::from_xyz(plate.center().x, plate.center().y, PLATE_Z),
    ));

    for index in 0..count {
        let at = switch_at(panel.room, count, index);

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
                room: panel.room,
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
            Led { room: panel.room },
            Sprite {
                color: LED_DARK_COLOR,
                custom_size: Some(LED_SIZE),
                ..default()
            },
            Transform::from_xyz(at.x, mount.y + LED_HEIGHT, FITTING_Z),
        ));
    }
}

/// Throws the nearest switch the player is standing at, across every room's
/// panel, and reads that panel afterwards to see whether that was the last
/// setting it was waiting for.
///
/// Nearest rather than first, for the same reason the doors do it: the player
/// means the switch they are in front of. Rooms are physically apart in world
/// space, so "nearest in reach" never crosses from one room's panel into
/// another's.
pub fn flip_switches(
    keys: Res<ButtonInput<KeyCode>>,
    mut panels: ResMut<Panels>,
    players: Query<&Transform, With<Player>>,
    mut switches: Query<(Entity, &mut Switch), Without<Player>>,
) {
    if !keys.just_pressed(KeyCode::KeyE) {
        return;
    }

    let Ok(player) = players.single() else {
        return;
    };
    let at = player.translation.truncate();

    let nearest = switches
        .iter()
        .filter(|(_, switch)| {
            // A solved panel is done with. Leaving it live would let a player
            // throw the combination back out again by walking past it.
            let count = panel_switch_count(&panels, switch.room);

            !panels.of(switch.room).solved
                && in_reach(
                    switch_at(switch.room, count, switch.index),
                    at,
                    switch_spacing(count),
                )
        })
        .min_by(|(_, one), (_, other)| {
            let distance = |switch: &Switch| {
                let count = panel_switch_count(&panels, switch.room);

                switch_at(switch.room, count, switch.index).distance_squared(at)
            };

            distance(one).total_cmp(&distance(other))
        })
        .map(|(entity, _)| entity);

    let Some(entity) = nearest else {
        return;
    };

    let room = switches
        .get(entity)
        .map(|(_, switch)| switch.room)
        .expect("the entity picked above came from this query");

    if let Ok((_, mut switch)) = switches.get_mut(entity) {
        switch.on = !switch.on;
    }

    let mut thrown = vec![false; panels.of(room).combination.len()];
    for (_, switch) in &switches {
        if switch.room == room
            && let Some(state) = thrown.get_mut(switch.index)
        {
            *state = switch.on;
        }
    }

    let panel = panels.of_mut(room);
    if panel.matched(&thrown) {
        panel.solved = true;
    }
}

/// How many switches the room's panel actually has, read off the resource
/// rather than recomputed, so a change to the difficulty curve cannot leave
/// the reach check out of step with what was spawned.
fn panel_switch_count(panels: &Panels, room: Room) -> usize {
    panels.of(room).combination.len()
}

/// Whether a player standing at `player` is at the switch drawn at `switch`,
/// given the spacing the switch's own panel was built with.
/// Measured off the middle of the slot rather than the toggle in it, which
/// moves.
fn in_reach(switch: Vec2, player: Vec2, spacing: f32) -> bool {
    let offset = (player - switch).abs();
    let bound = reach(spacing);

    offset.x <= bound.x && offset.y <= bound.y
}

/// Draws what every panel currently is: each toggle thrown its way, and each
/// panel's lamps lit or dark together with the rest of that panel.
///
/// The filters are what they are because a toggle, a lamp and the plate are all
/// sprites: each query has to say which of the three it means.
#[allow(clippy::type_complexity)]
pub fn light_panel(
    panels: Res<Panels>,
    mut toggles: Query<(&Switch, &mut Sprite, &mut Transform)>,
    mut leds: Query<(&Led, &mut Sprite), (Without<Switch>, Without<Backplate>)>,
    mut plates: Query<(&Backplate, &mut Sprite), (Without<Switch>, Without<Led>)>,
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

    for (led, mut sprite) in &mut leds {
        sprite.color = if panels.of(led.room).solved {
            LED_LIT_COLOR
        } else {
            LED_DARK_COLOR
        };
    }

    for (plate, mut sprite) in &mut plates {
        sprite.color = if panels.of(plate.room).solved {
            PLATE_SOLVED_COLOR
        } else {
            PLATE_COLOR
        };
    }
}

/// Spawned as part of the HUD, so it lives and dies with the rest of the run.
pub fn spawn_panel_status(
    parent: &mut ChildSpawnerCommands,
    panels: &Panels,
    progress: &LevelProgress,
    font_size: f32,
) {
    parent.spawn((
        PanelStatus,
        Text::new(panels.status(progress)),
        TextFont {
            font_size: FontSize::Px(font_size),
            ..default()
        },
        TextColor(MUTED_TEXT),
    ));
}

pub fn sync_panel_status(
    panels: Res<Panels>,
    progress: Res<LevelProgress>,
    mut labels: Query<&mut Text, With<PanelStatus>>,
) {
    if !panels.is_changed() && !progress.is_changed() {
        return;
    }

    for mut text in &mut labels {
        **text = panels.status(&progress);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::level::ROOMS_PER_DECK;

    /// The whole test module works against a fixed deck count, standing in
    /// for whichever difficulty a real run picks.
    const TEST_DECK_COUNT: usize = 4;
    const TEST_ROOM_COUNT: usize = TEST_DECK_COUNT * ROOMS_PER_DECK;

    /// Every room's panel has to ask for a workable setting — never all-down,
    /// which would be a panel solved before it was found.
    #[test]
    fn every_panel_asks_for_a_workable_combination() {
        for seed in 0..500u64 {
            let panels = Panels::from_seed(seed, Level::Rocket, TEST_DECK_COUNT);

            for panel in panels.iter() {
                assert!(
                    panel.combination.iter().any(|up| *up),
                    "the panel in {} was spawned already solved",
                    panel.room.label()
                );
            }
        }
    }

    /// Rooms further into the rocket carry more switches, up to the cap.
    #[test]
    fn switch_count_climbs_a_switch_a_deck_up_to_the_cap() {
        for index in 0..TEST_ROOM_COUNT {
            let room = Room::from_index(index);
            assert_eq!(switch_count(room), (3 + room.deck).min(4));
        }
    }

    /// Two runs a moment apart must not deal the same panels.
    #[test]
    fn seeds_a_moment_apart_give_different_panels() {
        let seed = 1_234_567_890_u64;
        let differ = (1..=8).filter(|step| {
            Panels::from_seed(seed + step, Level::Rocket, TEST_DECK_COUNT)
                != Panels::from_seed(seed, Level::Rocket, TEST_DECK_COUNT)
        });

        assert!(differ.count() >= 6, "the pick barely moves between seeds");
    }

    #[test]
    fn a_panel_starts_unsolved_and_needs_its_own_combination() {
        let panels = Panels::from_seed(7, Level::Rocket, TEST_DECK_COUNT);
        let panel = panels.of(Room::from_index(0));

        assert!(!panel.solved);
        assert!(!panel.matched(&vec![false; panel.combination.len()]));
        assert!(panel.matched(&panel.combination));
    }

    /// A player standing at a switch has to be able to reach it, and a player at
    /// the far end of the panel must not reach the one at the other end.
    #[test]
    fn each_switch_is_worked_from_in_front_of_it() {
        let panels = Panels::from_seed(0, Level::Rocket, TEST_DECK_COUNT);
        let panel = panels.of(Room::from_index(0));
        let count = panel.combination.len();
        let standing = working_positions(panel);

        for (index, at) in standing.iter().enumerate() {
            assert!(
                in_reach(
                    switch_at(panel.room, count, index),
                    *at,
                    switch_spacing(count)
                ),
                "switch {index} cannot be worked from in front of it"
            );
        }

        assert!(
            !in_reach(
                switch_at(panel.room, count, 0),
                standing[count - 1],
                switch_spacing(count)
            ),
            "the far switch is worked from the other end of the panel"
        );
    }

    /// `E` works a door and throws a switch, so the two must never be in reach
    /// at once — a press meant for one of them landing on both would open the
    /// bulkhead every time the player touched a panel.
    #[test]
    fn a_panel_is_never_in_reach_of_a_door() {
        let doors = Level::Rocket.doors(TEST_DECK_COUNT);

        for index in 0..TEST_ROOM_COUNT {
            let room = Room::from_index(index);
            let panel = Panel {
                room,
                ..Panel::for_room(room, 0)
            };

            for at in working_positions(&panel) {
                for door in &doors {
                    assert!(
                        !door.in_reach(at),
                        "the panel in {} is worked from the same spot as {door:?}",
                        panel.room.label()
                    );
                }
            }
        }
    }

    /// The other things in a room: a panel must not be drawn over a ladder or
    /// through a wall, whichever room it turns up in and however many switches
    /// its difficulty gives it.
    #[test]
    fn a_panel_is_clear_of_the_ladders_and_the_hull() {
        let ladders = Level::Rocket.ladders(TEST_DECK_COUNT);
        let walls = Level::Rocket.walls(TEST_DECK_COUNT);

        for index in 0..TEST_ROOM_COUNT {
            let room = Room::from_index(index);
            let panel = Panel {
                room,
                ..Panel::for_room(room, 0)
            };
            let plate = plate_bounds(&panel);

            for ladder in &ladders {
                let column = ladder.reach();
                let clear = plate.max.x <= column.min.x || plate.min.x >= column.max.x;

                assert!(
                    clear,
                    "the panel in {} ({} switches) is hung over the ladder at x={}",
                    panel.room.label(),
                    panel.combination.len(),
                    ladder.x
                );
            }

            for wall in &walls {
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

        for index in 0..TEST_ROOM_COUNT {
            let room = Room::from_index(index);
            let panel = Panel {
                room,
                ..Panel::for_room(room, 0)
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

    /// A panel is never mounted on top of its own room's breach: the two are on
    /// different stretches of wall so working one never hides the other.
    ///
    /// Except in the bottom deck's port room, where the airlock's reach rules
    /// out every other stretch of wall — see [`Room::panel_mount`] — and that
    /// panel is deliberately mounted alongside its breach instead, the way the
    /// single panel a run once dealt always was.
    #[test]
    fn a_panel_never_overlaps_its_own_room_s_breach() {
        use crate::level::Side;
        use crate::portal::PORTAL_RADIUS;

        for index in 0..TEST_ROOM_COUNT {
            let room = Room::from_index(index);
            if room.deck == 0 && room.side == Side::Port {
                continue;
            }

            let panel = Panel {
                room,
                ..Panel::for_room(room, 0)
            };
            let plate = plate_bounds(&panel);
            let breach = room.portal_mount();

            let clear =
                breach.x + PORTAL_RADIUS <= plate.min.x || breach.x - PORTAL_RADIUS >= plate.max.x;

            assert!(
                clear,
                "the panel in {} overlaps its own breach",
                room.label()
            );
        }
    }

    /// Every panel is always somewhere the run goes: one in every room of the
    /// rocket, at whatever deck count the run was dealt.
    #[test]
    fn there_is_a_panel_in_every_room_of_the_run() {
        for seed in 0..64u64 {
            let panels = Panels::from_seed(seed, Level::Rocket, TEST_DECK_COUNT);

            for room in Level::Rocket.rooms(TEST_DECK_COUNT) {
                assert_eq!(panels.of(room).room, room);
            }
        }
    }

    /// The panel as it is actually worked: a real player entity walked from one
    /// switch to the next, pressing `E`, against the same systems the run uses.
    /// The layout tests above say the fittings are in the right places; these say
    /// that throwing them does something.
    mod working_it {
        use super::*;

        struct Bench {
            app: App,
            player: Entity,
        }

        impl Bench {
            fn with(panels: Panels) -> Self {
                let mut app = App::new();
                app.add_plugins(MinimalPlugins);
                app.insert_resource(panels);
                app.insert_resource(ButtonInput::<KeyCode>::default());
                app.add_systems(Startup, |mut commands: Commands, panels: Res<Panels>| {
                    spawn_panels(&mut commands, &panels, Level::Rocket, TEST_DECK_COUNT, ());
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

            fn panel(&self, room: Room) -> Panel {
                self.app.world().resource::<Panels>().of(room).clone()
            }

            fn switches(&mut self, room: Room) -> Vec<bool> {
                let mut query = self.app.world_mut().query::<&Switch>();

                query
                    .iter(self.app.world())
                    .filter(|switch| switch.room == room)
                    .map(|switch| (switch.index, switch.on))
                    .collect::<std::collections::BTreeMap<_, _>>()
                    .into_values()
                    .collect()
            }

            fn lamps_lit(&mut self, room: Room) -> bool {
                let mut query = self.app.world_mut().query::<&Led>();
                let mut lamps = query
                    .iter(self.app.world())
                    .filter(|led| led.room == room)
                    .peekable();

                lamps.peek().is_some() && {
                    let panels = self.app.world().resource::<Panels>();
                    panels.of(room).solved
                }
            }
        }

        #[test]
        fn a_panel_is_built_with_as_many_switches_and_lamps_as_its_difficulty() {
            let panels = Panels::from_seed(3, Level::Rocket, TEST_DECK_COUNT);
            let room = Room::from_index(0);
            let count = panels.of(room).combination.len();
            let mut bench = Bench::with(panels);

            assert_eq!(bench.switches(room), vec![false; count]);
            assert!(
                !bench.lamps_lit(room),
                "the lamps are lit on an unsolved panel"
            );
        }

        #[test]
        fn a_press_at_a_switch_throws_that_switch_and_no_other() {
            let panels = Panels::from_seed(3, Level::Rocket, TEST_DECK_COUNT);
            let room = Room::from_index(0);
            let standing = working_positions(panels.of(room));
            let mut bench = Bench::with(panels);

            bench.throw(standing[1]);

            let switches = bench.switches(room);
            assert!(switches[1]);
            assert!(switches.iter().enumerate().all(|(i, on)| i == 1 || !on));
        }

        #[test]
        fn a_press_away_from_the_panel_throws_nothing() {
            let panels = Panels::from_seed(3, Level::Rocket, TEST_DECK_COUNT);
            let room = Room::from_index(0);
            let count = panels.of(room).combination.len();
            let mut bench = Bench::with(panels);

            bench.throw(room.panel_mount() + Vec2::new(400.0, 0.0));

            assert_eq!(bench.switches(room), vec![false; count]);
        }

        /// The whole of it: work the switches into the combination and the panel
        /// solves and the lamps come on. Run over every seed's worth of room and
        /// combination, so it is not one lucky layout that works.
        #[test]
        fn setting_the_combination_solves_the_room_and_lights_the_lamps() {
            for seed in 0..24u64 {
                let panels = Panels::from_seed(seed, Level::Rocket, TEST_DECK_COUNT);
                let room = Room::from_index(0);
                let combination = panels.of(room).combination.clone();
                let standing = working_positions(panels.of(room));
                let mut bench = Bench::with(panels);

                for (index, wanted) in combination.iter().enumerate() {
                    if *wanted {
                        bench.throw(standing[index]);
                    }
                }

                assert_eq!(bench.switches(room), combination);
                assert!(
                    bench.panel(room).solved,
                    "the panel in {} was set and not solved",
                    room.label()
                );
                assert!(
                    bench.lamps_lit(room),
                    "the lamps stayed dark on a solved panel"
                );
            }
        }

        /// Anything short of the combination leaves the room unsolved — including
        /// every switch up, which is the setting a player who simply throws every
        /// switch would land in.
        #[test]
        fn a_partial_or_wrong_setting_leaves_the_room_unsolved() {
            let room = Room::from_index(0);
            let mut panels = Panels::from_seed(0, Level::Rocket, TEST_DECK_COUNT);
            *panels.of_mut(room) = Panel {
                combination: vec![true, false, true],
                ..panels.of(room).clone()
            };
            let standing = working_positions(panels.of(room));

            let mut bench = Bench::with(panels.clone());
            bench.throw(standing[0]);
            assert!(!bench.panel(room).solved, "one switch of three solved it");
            assert!(!bench.lamps_lit(room));

            let mut bench = Bench::with(panels);
            for at in standing {
                bench.throw(at);
            }
            assert_eq!(bench.switches(room), vec![true, true, true]);
            assert!(!bench.panel(room).solved, "every switch up solved it");
            assert!(!bench.lamps_lit(room), "the lamps lit on a wrong setting");
        }

        /// Once solved it stays solved: the switches lock, so a player crossing
        /// the room again cannot throw the combination back out.
        #[test]
        fn a_solved_panel_cannot_be_thrown_back_out() {
            let room = Room::from_index(0);
            let mut panels = Panels::from_seed(0, Level::Rocket, TEST_DECK_COUNT);
            *panels.of_mut(room) = Panel {
                combination: vec![false, true, false],
                ..panels.of(room).clone()
            };
            let standing = working_positions(panels.of(room));
            let mut bench = Bench::with(panels);

            bench.throw(standing[1]);
            assert!(bench.panel(room).solved);

            bench.throw(standing[1]);

            assert_eq!(bench.switches(room), vec![false, true, false]);
            assert!(bench.panel(room).solved);
            assert!(bench.lamps_lit(room));
        }
    }
}
