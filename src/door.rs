//! The doors between the rocket's rooms, and the airlock out of it.
//!
//! A door is shut until it is worked: it carries a collider that fills its
//! opening, and pressing `E` in front of it takes that collider away and
//! retracts the panel into the lintel. Doors stay open once opened — the run is
//! against a clock, and a door that shut behind the player would only ever cost
//! them the same press twice.
//!
//! One door in the rocket is not a way between rooms but the way out. It is
//! worked like any other, and since working a door means standing at it, the
//! press that opens the airlock is in practice the same one that puts the run
//! outside. Opening it and leaving through it are still two systems rather than
//! one, so that an airlock already open — walked away from and come back to —
//! puts the player out just the same.

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::config::{PLAYER_HEIGHT, PLAYER_WIDTH, WALL_THICKNESS};
use crate::level::{Level, LevelProgress};
use crate::panel::Panel;
use crate::player::Player;
use crate::props::LARGEST_CRATE_SIZE;
use crate::settings::Settings;
use crate::state::PlayingState;

/// As thick as the bulkhead it is set into, so the panel fills the wall rather
/// than standing proud of it, and tall enough to walk through without ducking.
pub const DOOR_SIZE: Vec2 = Vec2::new(WALL_THICKNESS, 96.0);

/// How far the panel leaves showing once it is open — the lip of it, tucked up
/// under the lintel, so an opened door is still visibly a door.
const OPEN_LIP: f32 = 10.0;

const SHUT_COLOR: Color = Color::srgb(0.62, 0.42, 0.22);
const OPEN_COLOR: Color = Color::srgb(0.38, 0.30, 0.22);
/// Worn by the airlock while it is still held shut, so a door that will not
/// open says why before the player has pressed anything.
const AIRLOCK_BLOCKED_COLOR: Color = Color::srgb(0.85, 0.28, 0.25);
/// The way out is coloured apart from the bulkhead doors, so the room the run
/// ends in says so before the player has walked the length of it.
const AIRLOCK_COLOR: Color = Color::srgb(0.45, 0.9, 0.55);

/// Behind the player, so walking into an opened door reads as stepping through
/// it, and behind the bulkhead tiles it is set into.
const DOOR_Z: f32 = -2.0;

/// A little further than any of the arithmetic strictly needs, so the check
/// never lands exactly on the position a player pressed up against something
/// ends up in — a boundary no float should be asked to sit on.
const REACH_MARGIN: f32 = 8.0;

/// How far off a door's centre still counts as standing at it.
///
/// Not simply where the player and the panel touch, which is what this was at
/// first and what made doors look broken. A player walking a deck pushes
/// whatever is loose on it along ahead of them, so they arrive at the door with
/// a crate wedged between them and it, standing a crate's width further back
/// than the panel's face. With the tighter reach that put the door out of range
/// at exactly the moment the player was trying to open it, and the run
/// dead-ended: the crate cannot go through a shut door, and the door could not
/// be worked past the crate. So the reach is measured over the top of the
/// largest crate that could be leaning on it.
///
/// Vertically there is no such problem — nothing is going to end up stacked
/// between the player and a doorway they are standing in — so that stays at
/// where the two touch, which is what keeps a door on the deck below from being
/// worked through the floor.
const REACH: Vec2 = Vec2::new(
    DOOR_SIZE.x / 2.0 + LARGEST_CRATE_SIZE + PLAYER_WIDTH / 2.0 + REACH_MARGIN,
    (DOOR_SIZE.y + PLAYER_HEIGHT) / 2.0,
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoorKind {
    /// Between two rooms on the same deck.
    Bulkhead,
    /// Out of the rocket altogether.
    Airlock,
}

/// A door, as both the layout that describes it and the component that tracks
/// it once it is standing. The two are the same type because a level's doors
/// are written down as constants and then spawned unchanged.
#[derive(Component, Debug, Clone, Copy)]
pub struct Door {
    pub at: Vec2,
    pub kind: DoorKind,
    pub open: bool,
    pub locked: bool,
}

impl Door {
    /// Built from the deck plate it stands on rather than its own centre, so a
    /// layout says which floor a door is on and the sill looks after itself.
    const fn standing_on(x: f32, deck: f32, kind: DoorKind) -> Self {
        Self {
            at: Vec2::new(x, deck + DOOR_SIZE.y / 2.0),
            kind,
            open: false,
            locked: false,
        }
    }

    pub const fn bulkhead(x: f32, deck: f32) -> Self {
        Self::standing_on(x, deck, DoorKind::Bulkhead)
    }

    pub const fn airlock(x: f32, deck: f32) -> Self {
        Self {
            locked: true,
            ..Self::standing_on(x, deck, DoorKind::Airlock)
        }
    }

    /// The sill — the deck plate the door stands on.
    pub const fn sill(&self) -> f32 {
        self.at.y - DOOR_SIZE.y / 2.0
    }

    /// The head of the doorway. What a bulkhead is built off: the wall picks up
    /// where the door leaves off, so the two cannot drift apart and leave either
    /// a gap over the door or a lintel sitting across it.
    pub const fn lintel(&self) -> f32 {
        self.sill() + DOOR_SIZE.y
    }

    pub fn in_reach(&self, player: Vec2) -> bool {
        let offset = (player - self.at).abs();

        offset.x <= REACH.x && offset.y <= REACH.y
    }

    pub fn color(&self) -> Color {
        match self.kind {
            DoorKind::Airlock if self.locked => AIRLOCK_BLOCKED_COLOR,
            DoorKind::Airlock => AIRLOCK_COLOR,
            DoorKind::Bulkhead if self.open => OPEN_COLOR,
            DoorKind::Bulkhead => SHUT_COLOR,
        }
    }
}

pub fn spawn_doors(
    commands: &mut Commands,
    doors: &[Door],
    marker: impl Bundle + Clone,
    level: Level,
    deck_count: usize,
    panel: Panel,
    progress: crate::level::LevelProgress,
) {
    for door in doors {
        let mut door = *door;

        if door.kind == DoorKind::Airlock {
            door.locked =
                !progress.all_obstacles_completed(level, deck_count, panel.room, panel.solved);
        }

        commands.spawn((
            marker.clone(),
            door,
            Sprite {
                color: door.color(),
                custom_size: Some(DOOR_SIZE),
                ..default()
            },
            Transform::from_xyz(door.at.x, door.at.y, DOOR_Z),
            RigidBody::Fixed,
            Collider::cuboid(DOOR_SIZE.x / 2.0, DOOR_SIZE.y / 2.0),
        ));
    }
}

/// Keeps the airlock aligned with what the run has actually done: locked
/// until every obstacle on the level — the panel and every breach — is
/// signed off.
pub fn sync_airlock_lock_state(
    level: Res<Level>,
    settings: Res<Settings>,
    panel: Res<Panel>,
    progress: Res<LevelProgress>,
    mut doors: Query<(&mut Door, &mut Sprite)>,
) {
    if !level.is_changed() && !panel.is_changed() && !progress.is_changed() {
        return;
    }

    let deck_count = settings.difficulty.deck_count();
    let locked = !progress.all_obstacles_completed(*level, deck_count, panel.room, panel.solved);

    for (mut door, mut sprite) in &mut doors {
        if door.kind != DoorKind::Airlock {
            continue;
        }

        if door.locked != locked {
            door.locked = locked;
            sprite.color = door.color();
        }
    }
}

/// Works the nearest shut door the player is standing at.
///
/// Nearest rather than first, because the layout is free to put a door on each
/// side of a small room and the player means the one they are up against.
pub fn use_doors(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    players: Query<&Transform, With<Player>>,
    mut doors: Query<(Entity, &mut Door, &mut Sprite, &mut Transform), Without<Player>>,
) {
    if !keys.just_pressed(KeyCode::KeyE) {
        return;
    }

    let Ok(player) = players.single() else {
        return;
    };
    let at = player.translation.truncate();

    let nearest = doors
        .iter()
        .filter(|(_, door, _, _)| !door.open && !door.locked && door.in_reach(at))
        .min_by(|(_, one, _, _), (_, other, _, _)| {
            one.at
                .distance_squared(at)
                .total_cmp(&other.at.distance_squared(at))
        })
        .map(|(entity, ..)| entity);

    let Some(entity) = nearest else {
        return;
    };
    let Ok((_, mut door, mut sprite, mut transform)) = doors.get_mut(entity) else {
        return;
    };

    door.open = true;
    sprite.color = door.color();
    sprite.custom_size = Some(Vec2::new(DOOR_SIZE.x, OPEN_LIP));
    // Slid up until only the lip is left showing under the lintel. The door's
    // own `at` is left alone: it is where the doorway is, not where the panel
    // happens to have got to.
    transform.translation.y = door.at.y + (DOOR_SIZE.y - OPEN_LIP) / 2.0;

    // Taking the collider off is what actually opens the way through. The
    // entity keeps its body, so nothing else has to be rebuilt.
    commands.entity(entity).remove::<Collider>();
}

/// Ends the run when the player steps into the opened airlock. It is the way
/// back out of the rocket and there is nothing beyond it, so stepping through
/// is the mission finished rather than the next level starting.
pub fn leave_through_airlock(
    players: Query<&Transform, With<Player>>,
    doors: Query<&Door>,
    mut next_playing: ResMut<NextState<PlayingState>>,
) {
    let Ok(player) = players.single() else {
        return;
    };
    let at = player.translation.truncate();

    let stepped_out = doors.iter().any(|door| {
        door.kind == DoorKind::Airlock && door.open && !door.locked && door.in_reach(at)
    });

    if stepped_out {
        next_playing.set(PlayingState::MissionComplete);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{PIXELS_PER_METER, PLATFORM_HEIGHT};
    use crate::physics::configure_physics;
    use crate::player::{move_player, physics_body};
    use std::time::Duration;

    const DECK: f32 = -360.0;
    const TEST_DECK_COUNT: usize = 4;

    /// The way out says whether it is a way out yet: held shut and red while
    /// there is still a job on the level, green the moment the last of them is
    /// signed off. It is across the rocket from most of the work, so the colour
    /// is what tells a player they are done rather than the walk over to it.
    #[test]
    fn the_airlock_goes_green_once_every_job_is_signed_off() {
        let mut app = App::new();
        app.init_resource::<Level>();
        app.init_resource::<Settings>();
        app.init_resource::<Panel>();
        app.insert_resource(LevelProgress::new(Level::Rocket, TEST_DECK_COUNT));
        app.add_systems(Update, sync_airlock_lock_state);

        let airlock = Door::airlock(0.0, DECK);
        let door = app
            .world_mut()
            .spawn((
                airlock,
                Sprite {
                    color: airlock.color(),
                    ..default()
                },
            ))
            .id();

        app.update();
        assert_eq!(
            app.world().entity(door).get::<Sprite>().unwrap().color,
            AIRLOCK_BLOCKED_COLOR,
            "the airlock is green with the whole level still to work"
        );

        // Sign off the panel and every breach.
        app.world_mut().resource_mut::<Panel>().solved = true;
        let mut progress = app.world_mut().resource_mut::<LevelProgress>();
        progress.completed_portals = progress.total_portals;
        app.update();

        assert!(!app.world().entity(door).get::<Door>().unwrap().locked);
        assert_eq!(
            app.world().entity(door).get::<Sprite>().unwrap().color,
            AIRLOCK_COLOR,
            "the airlock did not go green once the level was clear"
        );
    }

    /// The way out is the way in: a run opens with the player stood at it.
    #[test]
    fn the_airlock_is_at_the_entrance_the_run_opens_at() {
        let airlock = Level::Rocket
            .doors(TEST_DECK_COUNT)
            .into_iter()
            .find(|door| door.kind == DoorKind::Airlock)
            .expect("the rocket has no airlock to leave by");
        let spawn = Level::Rocket.player_spawn();

        assert!(
            spawn.y > airlock.sill() && spawn.y < airlock.lintel(),
            "the run opens on a different deck from the airlock"
        );
        assert!(
            airlock.in_reach(spawn),
            "the run does not open within reach of the airlock it boarded through"
        );
        assert!(
            airlock.locked,
            "the airlock starts unlocked rather than shut until the run is worked"
        );
    }

    #[test]
    fn a_door_stands_on_the_deck_it_is_given() {
        assert_eq!(Door::bulkhead(0.0, DECK).sill(), DECK);
        assert_eq!(Door::airlock(580.0, DECK).sill(), DECK);
    }

    /// The press has to land from where a player pressed up against a shut door
    /// actually stands, which is their half-width off the panel's face.
    #[test]
    fn a_player_up_against_a_door_can_work_it() {
        let door = Door::bulkhead(0.0, DECK);
        let touching = (DOOR_SIZE.x + PLAYER_WIDTH) / 2.0;
        let standing = Vec2::new(touching, DECK + PLAYER_HEIGHT / 2.0);

        assert!(door.in_reach(standing));
        assert!(door.in_reach(Vec2::new(-touching, standing.y)));
    }

    /// The regression that made doors look broken: walking a deck shoves the
    /// crates on it along ahead of you, so you arrive at the door standing a
    /// crate's width back from it — and a reach measured to the panel's face
    /// leaves you unable to open the one thing you are stood in front of.
    #[test]
    fn a_crate_shoved_against_a_door_does_not_shut_the_player_out() {
        let door = Door::bulkhead(0.0, DECK);
        let behind_the_crate = DOOR_SIZE.x / 2.0 + LARGEST_CRATE_SIZE + PLAYER_WIDTH / 2.0;
        let standing = Vec2::new(behind_the_crate, DECK + PLAYER_HEIGHT / 2.0);

        assert!(door.in_reach(standing));
        assert!(door.in_reach(Vec2::new(-behind_the_crate, standing.y)));
    }

    #[test]
    fn a_door_across_the_room_is_out_of_reach() {
        let door = Door::bulkhead(0.0, DECK);

        assert!(!door.in_reach(Vec2::new(400.0, DECK + PLAYER_HEIGHT / 2.0)));
    }

    /// A door on the deck below must not be worked through the floor by someone
    /// standing on top of it.
    #[test]
    fn a_door_on_another_deck_is_out_of_reach() {
        let door = Door::bulkhead(0.0, DECK);
        let deck_above = DECK + 260.0 + PLAYER_HEIGHT / 2.0;

        assert!(!door.in_reach(Vec2::new(0.0, deck_above)));
    }

    #[test]
    fn the_airlock_is_coloured_apart_from_the_bulkhead_doors() {
        assert_ne!(
            Door::airlock(0.0, DECK).color(),
            Door::bulkhead(0.0, DECK).color()
        );
    }

    const STEP: f32 = 1.0 / 60.0;
    const BULKHEAD_DOOR: Door = Door::bulkhead(0.0, DECK);

    /// Walks a player into a shut door, works it, and walks them on through.
    /// Returns where they got to on each of the three legs.
    ///
    /// A door is only worth anything if it actually stops someone, so this is
    /// run against Rapier rather than against `in_reach` on its own: the first
    /// leg is the one that would quietly pass if the collider were never there.
    fn walk_into_the_door_then_open_it() -> (f32, f32) {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            TransformPlugin,
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(PIXELS_PER_METER),
        ));
        app.insert_resource(TimestepMode::Fixed {
            dt: STEP,
            substeps: 1,
        });
        app.insert_resource(ButtonInput::<KeyCode>::default());
        app.add_systems(Startup, configure_physics);
        app.add_systems(Startup, |mut commands: Commands| {
            spawn_doors(
                &mut commands,
                &[BULKHEAD_DOOR],
                (),
                Level::Rocket,
                TEST_DECK_COUNT,
                Panel::default(),
                LevelProgress::new(Level::Rocket, TEST_DECK_COUNT),
            );
        });
        app.add_systems(Update, (move_player, use_doors).chain());

        // The deck the door stands on, so the player has something to walk along.
        app.world_mut().spawn((
            Transform::from_xyz(0.0, DECK - PLATFORM_HEIGHT / 2.0, 0.0),
            RigidBody::Fixed,
            Collider::cuboid(600.0, PLATFORM_HEIGHT / 2.0),
        ));

        let player = app
            .world_mut()
            .spawn((
                Player,
                Transform::from_xyz(-300.0, DECK + PLAYER_HEIGHT / 2.0, 0.0),
                physics_body(),
            ))
            .id();

        // Reports where the player got to, so the caller never has to reach
        // into the world while this still has hold of it.
        let mut run = |steps: usize, held: &[KeyCode]| {
            let mut keys = ButtonInput::<KeyCode>::default();
            for key in held {
                keys.press(*key);
            }
            app.world_mut().insert_resource(keys);

            for _ in 0..steps {
                app.world_mut()
                    .resource_mut::<Time>()
                    .advance_by(Duration::from_secs_f32(STEP));
                app.update();
            }

            app.world()
                .entity(player)
                .get::<Transform>()
                .unwrap()
                .translation
                .x
        };

        // Long enough to cross the 300 units to the door at PLAYER_SPEED and
        // then some, so being stopped is the door's doing and not the clock's.
        let against_the_door = run(90, &[KeyCode::KeyD]);

        run(2, &[KeyCode::KeyE]);
        let through_the_door = run(60, &[KeyCode::KeyD]);

        (against_the_door, through_the_door)
    }

    /// Builds the rocket's real geometry — its own plates, hull, bulkheads and
    /// doors, straight off the level — as colliders, with no art and no asset
    /// server. The synthetic test above proves the mechanic; this one proves the
    /// layout the player actually gets, which is where an unreachable door or a
    /// doorway walled off by its own bulkhead would show up.
    fn spawn_rocket_geometry(app: &mut App) {
        let level = Level::Rocket;

        for plate in level.platforms(TEST_DECK_COUNT) {
            app.world_mut().spawn((
                Transform::from_xyz(plate.centre.x, plate.centre.y, 0.0),
                RigidBody::Fixed,
                Collider::cuboid(plate.width / 2.0, PLATFORM_HEIGHT / 2.0),
            ));
        }

        for wall in level.walls(TEST_DECK_COUNT) {
            let half_extents = wall.axis.half_extents(wall.length, WALL_THICKNESS);

            app.world_mut().spawn((
                Transform::from_xyz(wall.centre.x, wall.centre.y, 0.0),
                RigidBody::Fixed,
                Collider::cuboid(half_extents.x, half_extents.y),
            ));
        }

        app.add_systems(Startup, |mut commands: Commands| {
            spawn_doors(
                &mut commands,
                &Level::Rocket.doors(TEST_DECK_COUNT),
                (),
                Level::Rocket,
                TEST_DECK_COUNT,
                Panel::default(),
                LevelProgress::new(Level::Rocket, TEST_DECK_COUNT),
            );
        });
    }

    /// Walks the player out of their drop point into the bottom deck's bulkhead
    /// door, works it, and carries on into the room beyond.
    #[test]
    fn the_bulkhead_door_the_player_first_meets_can_be_worked() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            TransformPlugin,
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(PIXELS_PER_METER),
        ));
        app.insert_resource(TimestepMode::Fixed {
            dt: STEP,
            substeps: 1,
        });
        app.insert_resource(ButtonInput::<KeyCode>::default());
        app.add_systems(Startup, configure_physics);
        app.add_systems(Update, (move_player, use_doors).chain());
        spawn_rocket_geometry(&mut app);

        let spawn = Level::Rocket.player_spawn();
        let player = app
            .world_mut()
            .spawn((
                Player,
                Transform::from_xyz(spawn.x, spawn.y, 0.0),
                physics_body(),
            ))
            .id();

        let mut run = |steps: usize, held: &[KeyCode]| {
            let mut keys = ButtonInput::<KeyCode>::default();
            for key in held {
                keys.press(*key);
            }
            app.world_mut().insert_resource(keys);

            for _ in 0..steps {
                app.world_mut()
                    .resource_mut::<Time>()
                    .advance_by(Duration::from_secs_f32(STEP));
                app.update();
            }

            app.world()
                .entity(player)
                .get::<Transform>()
                .unwrap()
                .translation
                .x
        };

        // Ample time to cross the room and fetch up against the shut door.
        let against_the_door = run(180, &[KeyCode::KeyD]);
        assert!(
            against_the_door < 0.0,
            "walked straight through the bulkhead to x={against_the_door}"
        );

        run(2, &[KeyCode::KeyE]);
        let through_the_door = run(120, &[KeyCode::KeyD]);

        assert!(
            through_the_door > DOOR_SIZE.x,
            "the bulkhead door did not open: stuck at x={through_the_door}"
        );
    }

    #[test]
    fn a_shut_door_stops_the_player_and_an_opened_one_lets_them_by() {
        let (against_the_door, through_the_door) = walk_into_the_door_then_open_it();

        assert!(
            against_the_door < 0.0,
            "walked through a shut door to x={against_the_door}"
        );
        assert!(
            through_the_door > DOOR_SIZE.x,
            "still stuck at x={through_the_door} after opening the door"
        );
    }
}
