use bevy::prelude::*;

use crate::platform::spawn_platforms;
use crate::player::spawn_player;
use crate::props::spawn_props;
use crate::tiles::load_tiles;
use crate::ui::spawn_hud;
use crate::wall::spawn_walls;

/// Marks everything built for a single run, so quitting to the menu can clear
/// the level without disturbing the camera and backdrop spawned at startup.
#[derive(Component)]
pub struct GameEntity;

pub fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    let tiles = load_tiles(&assets);

    spawn_walls(&mut commands, &tiles);
    spawn_platforms(&mut commands, &tiles);
    spawn_player(&mut commands, &assets);
    spawn_props(&mut commands);
    spawn_hud(&mut commands);
}

pub fn despawn_game(mut commands: Commands, entities: Query<Entity, With<GameEntity>>) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
}
