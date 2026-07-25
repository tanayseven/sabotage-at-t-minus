use bevy::prelude::*;

use crate::door::spawn_doors;
use crate::ladder::spawn_ladders;
use crate::level::{Level, LevelEntity};
use crate::platform::spawn_platforms;
use crate::player::spawn_player;
use crate::props::spawn_props;
use crate::tiles::load_tiles;
use crate::ui::spawn_hud;
use crate::wall::spawn_wall_run;

/// Marks the parts of a run that outlive the level it is on — the HUD. Quitting
/// to the menu clears it without disturbing the camera and backdrop spawned at
/// startup.
#[derive(Component, Clone)]
pub struct GameEntity;

pub fn setup(mut commands: Commands, assets: Res<AssetServer>, level: Res<Level>) {
    build_level(&mut commands, &assets, *level);
    spawn_hud(&mut commands);
}

/// Everything belonging to one level, built from that level's own layout.
pub fn build_level(commands: &mut Commands, assets: &AssetServer, level: Level) {
    let tiles = load_tiles(assets);

    spawn_wall_run(commands, &tiles, level.walls(), LevelEntity);
    spawn_platforms(commands, &tiles, level.platforms(), LevelEntity);
    spawn_ladders(commands, assets, level.ladders(), LevelEntity);
    spawn_doors(commands, level.doors(), LevelEntity);
    spawn_props(commands, level.crates(), LevelEntity);
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
