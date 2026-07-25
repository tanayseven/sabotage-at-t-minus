use bevy::prelude::*;

use crate::countdown::MissionTimer;
use crate::level::{Level, LevelEntity, PendingLevelAdvance};
use crate::manual::ManualPage;
use crate::platform::spawn_platforms;
use crate::player::spawn_player;
use crate::portal::spawn_portal;
use crate::portal::PortalState;
use crate::props::spawn_props;
use crate::tiles::load_tiles;
use crate::ui::spawn_hud;

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
) {
    reset_run_state(&mut commands);
    build_level(&mut commands, &assets, &mut images, *level);
    spawn_hud(&mut commands);
}

fn reset_run_state(commands: &mut Commands) {
    commands.insert_resource(MissionTimer::default());
    commands.insert_resource(ManualPage::default());
    commands.insert_resource(PortalState::default());
}

/// Everything belonging to one level, built from that level's own layout.
pub fn build_level(
    commands: &mut Commands,
    assets: &AssetServer,
    images: &mut Assets<Image>,
    level: Level,
) {
    let tiles = load_tiles(assets);

    spawn_platforms(commands, &tiles, level.platforms(), LevelEntity);
    spawn_props(commands, level.crates(), LevelEntity);
    spawn_portal(commands, images, level, LevelEntity);
    spawn_player(commands, assets, level.player_spawn(), LevelEntity);
}

pub fn apply_pending_level_transition(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    pending: Option<Res<PendingLevelAdvance>>,
    hud: Query<Entity, With<GameEntity>>,
    level_entities: Query<Entity, With<LevelEntity>>,
) {
    let Some(pending) = pending else {
        return;
    };

    for entity in hud.iter().chain(&level_entities) {
        commands.entity(entity).despawn();
    }

    commands.insert_resource(pending.0);
    reset_run_state(&mut commands);
    build_level(&mut commands, &assets, &mut images, pending.0);
    spawn_hud(&mut commands);
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
