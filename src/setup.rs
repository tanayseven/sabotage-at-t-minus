use bevy::prelude::*;

use crate::countdown::MissionTimer;
use crate::door::spawn_doors;
use crate::ladder::spawn_ladders;
use crate::level::{Level, LevelEntity, LevelProgress, PendingLevelAdvance};
use crate::manual::ManualPage;
use crate::panel::{spawn_panel, Panel};
use crate::platform::spawn_platforms;
use crate::player::spawn_player;
use crate::portal::spawn_portals;
use crate::props::spawn_props;
use crate::tiles::load_tiles;
use crate::ui::spawn_hud;
use crate::wall::spawn_wall_run;

/// Marks the parts of a run that outlive the level it is on — the HUD. Quitting
/// to the menu clears it without disturbing the camera and backdrop spawned at
/// startup.
#[derive(Component, Clone)]
pub struct GameEntity;

pub fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    level: Res<Level>,
    time: Res<Time>,
) {
    let (panel, progress) = reset_run_state(&mut commands, *level, &time);
    build_level(&mut commands, &assets, &mut images, *level, panel, progress);
    spawn_hud(&mut commands, *level, &panel, &progress);
}

fn reset_run_state(commands: &mut Commands, level: Level, time: &Time) -> (Panel, LevelProgress) {
    commands.insert_resource(MissionTimer::default());
    commands.insert_resource(ManualPage::default());
    let panel = Panel::from_seed(time.elapsed().as_nanos() as u64);
    commands.insert_resource(panel);
    let progress = LevelProgress::new(level);
    commands.insert_resource(progress);
    (panel, progress)
}

/// Everything belonging to one level, built from that level's own layout.
pub fn build_level(
    commands: &mut Commands,
    assets: &AssetServer,
    images: &mut Assets<Image>,
    level: Level,
    panel: Panel,
    progress: LevelProgress,
) {
    let tiles = load_tiles(assets);

    spawn_wall_run(commands, &tiles, level.walls(), LevelEntity);
    spawn_platforms(commands, &tiles, level.platforms(), LevelEntity);
    spawn_ladders(commands, assets, level.ladders(), LevelEntity);
    spawn_doors(commands, level.doors(), LevelEntity, level, panel, progress);
    spawn_panel(commands, &panel, level, LevelEntity);
    spawn_props(commands, level.crates(), LevelEntity);
    spawn_portals(commands, images, level, LevelEntity);
    spawn_player(commands, assets, level.player_spawn(), LevelEntity);
}

pub fn apply_pending_level_transition(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    pending: Option<Res<PendingLevelAdvance>>,
    hud: Query<Entity, With<GameEntity>>,
    level_entities: Query<Entity, With<LevelEntity>>,
    time: Res<Time>,
) {
    let Some(pending) = pending else {
        return;
    };

    for entity in hud.iter().chain(&level_entities) {
        commands.entity(entity).despawn();
    }

    commands.insert_resource(pending.0);
    let (panel, progress) = reset_run_state(&mut commands, pending.0, &time);
    build_level(&mut commands, &assets, &mut images, pending.0, panel, progress);
    spawn_hud(&mut commands, pending.0, &panel, &progress);
    commands.remove_resource::<PendingLevelAdvance>();
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