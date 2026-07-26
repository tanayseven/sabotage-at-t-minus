use bevy::prelude::*;

use crate::countdown::MissionTimer;
use crate::door::spawn_doors;
use crate::ladder::spawn_ladders;
use crate::level::{Level, LevelEntity, LevelProgress};
use crate::manual::ManualPage;
use crate::panel::{Panel, spawn_panel};
use crate::platform::spawn_platforms;
use crate::player::spawn_player;
use crate::portal::spawn_portals;
use crate::props::spawn_props;
use crate::puzzles::RocketPuzzles;
use crate::tiles::load_tiles;
use crate::ui::spawn_hud;
use crate::wall::{spawn_hull_lining, spawn_wall_run};

/// Marks the parts of a run that outlive the level it is on — the HUD. Quitting
/// to the menu clears it without disturbing the camera and backdrop spawned at
/// startup.
#[derive(Component, Clone)]
pub struct GameEntity;

pub fn setup(mut commands: Commands, assets: Res<AssetServer>, level: Res<Level>, time: Res<Time>) {
    let run = reset_run_state(&mut commands, *level, &time);
    build_level(&mut commands, &assets, *level, run);
    spawn_hud(&mut commands, *level, &run.panel, &run.progress);
}

/// What one level of a run is built from: where the puzzles are, what the panel
/// wants, and how much of it is done.
#[derive(Clone, Copy)]
pub struct RunState {
    pub puzzles: RocketPuzzles,
    pub panel: Panel,
    pub progress: LevelProgress,
}

fn reset_run_state(commands: &mut Commands, level: Level, time: &Time) -> RunState {
    commands.insert_resource(MissionTimer::default());
    commands.insert_resource(ManualPage::default());

    // One seed for the whole deal, so the panel's room is drawn from the same
    // hand as the breaches' and no two of them land in one room.
    let seed = time.elapsed().as_nanos() as u64;
    let puzzles = RocketPuzzles::from_seed(seed);
    let panel = Panel::from_seed(seed);
    let progress = LevelProgress::new(level);

    commands.insert_resource(puzzles);
    commands.insert_resource(panel);
    commands.insert_resource(progress);

    RunState {
        puzzles,
        panel,
        progress,
    }
}

/// Everything belonging to one level, built from that level's own layout.
pub fn build_level(commands: &mut Commands, assets: &AssetServer, level: Level, run: RunState) {
    let tiles = load_tiles(assets);

    spawn_hull_lining(commands, assets, level.interior(), LevelEntity);
    spawn_wall_run(commands, &tiles, level.walls(), LevelEntity);
    spawn_platforms(commands, &tiles, level.platforms(), LevelEntity);
    spawn_ladders(commands, assets, level.ladders(), LevelEntity);
    spawn_doors(
        commands,
        level.doors(),
        LevelEntity,
        level,
        run.panel,
        run.progress,
    );
    spawn_panel(commands, &run.panel, level, LevelEntity);
    spawn_props(commands, level.crates(), LevelEntity);
    spawn_portals(commands, assets, run.puzzles, LevelEntity);
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
