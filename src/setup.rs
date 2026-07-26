use bevy::prelude::*;

use crate::countdown::MissionTimer;
use crate::door::spawn_doors;
use crate::ladder::spawn_ladders;
use crate::level::{Level, LevelEntity, LevelProgress, RoomCodes};
use crate::manual::ManualPage;
use crate::panel::{Panel, spawn_panel};
use crate::platform::spawn_platforms;
use crate::player::spawn_player;
use crate::portal::spawn_portals;
use crate::props::spawn_props;
use crate::puzzles::RocketPuzzles;
use crate::settings::Settings;
use crate::sign::spawn_room_signs;
use crate::tiles::load_tiles;
use crate::ui::{GameFont, spawn_hud};
use crate::wall::{spawn_hull_lining, spawn_wall_run};

/// Marks the parts of a run that outlive the level it is on — the HUD. Quitting
/// to the menu clears it without disturbing the camera and backdrop spawned at
/// startup.
#[derive(Component, Clone)]
pub struct GameEntity;

pub fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    level: Res<Level>,
    settings: Res<Settings>,
    time: Res<Time>,
    font: Res<GameFont>,
    codes: Res<RoomCodes>,
) {
    let deck_count = settings.difficulty.deck_count();
    let run = reset_run_state(&mut commands, *level, deck_count, &time);
    build_level(
        &mut commands,
        &assets,
        *level,
        deck_count,
        &codes,
        run.clone(),
    );
    spawn_hud(
        &mut commands,
        &font,
        &codes,
        *level,
        deck_count,
        &run.panel,
        &run.progress,
    );
}

/// What one level of a run is built from: where the puzzles are, what the panel
/// wants, and how much of it is done. Not `Copy`: the puzzles' room-by-room
/// deal is sized to the run's room count, so it is a `Vec` rather than a fixed
/// array.
#[derive(Clone)]
pub struct RunState {
    pub puzzles: RocketPuzzles,
    pub panel: Panel,
    pub progress: LevelProgress,
}

fn reset_run_state(
    commands: &mut Commands,
    level: Level,
    deck_count: usize,
    time: &Time,
) -> RunState {
    commands.insert_resource(MissionTimer::default());
    commands.insert_resource(ManualPage::default());

    let room_count = deck_count * crate::level::ROOMS_PER_DECK;

    // One seed for the whole deal, so the panel's room is drawn from the same
    // hand as the breaches' and no two of them land in one room.
    let seed = time.elapsed().as_nanos() as u64;
    let puzzles = RocketPuzzles::from_seed(seed, room_count);
    let panel = Panel::from_seed(seed, room_count);
    let progress = LevelProgress::new(level, deck_count);

    commands.insert_resource(puzzles.clone());
    commands.insert_resource(panel);
    commands.insert_resource(progress);

    RunState {
        puzzles,
        panel,
        progress,
    }
}

/// Everything belonging to one level, built from that level's own layout.
pub fn build_level(
    commands: &mut Commands,
    assets: &AssetServer,
    level: Level,
    deck_count: usize,
    codes: &RoomCodes,
    run: RunState,
) {
    let tiles = load_tiles(assets);

    let walls = level.walls(deck_count);
    let platforms = level.platforms(deck_count);
    let ladders = level.ladders(deck_count);
    let doors = level.doors(deck_count);
    let crates = level.crates(deck_count);

    spawn_hull_lining(commands, assets, level.interior(deck_count), LevelEntity);
    spawn_wall_run(commands, &tiles, &walls, LevelEntity);
    spawn_platforms(commands, &tiles, &platforms, LevelEntity);
    spawn_ladders(commands, assets, &ladders, LevelEntity);
    spawn_doors(
        commands,
        &doors,
        LevelEntity,
        level,
        deck_count,
        run.panel,
        run.progress,
    );
    spawn_panel(commands, &run.panel, level, deck_count, LevelEntity);
    spawn_props(commands, &crates, LevelEntity);
    spawn_portals(commands, assets, &run.puzzles, LevelEntity);
    spawn_room_signs(commands, codes, level, deck_count, LevelEntity);
    spawn_player(commands, assets, level.player_spawn(), LevelEntity);
}

pub fn despawn_game(
    mut commands: Commands,
    hud: Query<Entity, With<GameEntity>>,
    level: Query<Entity, With<LevelEntity>>,
) {
    for entity in hud.iter().chain(&level) {
        commands.entity(entity).despawn();
    }
}
