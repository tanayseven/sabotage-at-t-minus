//! Startup wiring: builds the scene by delegating to each module's spawner.

use bevy::prelude::*;

use crate::camera::spawn_camera;
use crate::platform::spawn_platforms;
use crate::player::spawn_player;
use crate::props::spawn_props;
use crate::ui::spawn_hud;
use crate::wall::spawn_walls;

/// Populates the world once at startup.
pub fn setup(mut commands: Commands) {
    spawn_camera(&mut commands);
    spawn_walls(&mut commands);
    spawn_platforms(&mut commands);
    spawn_player(&mut commands);
    spawn_props(&mut commands);
    spawn_hud(&mut commands);
}
