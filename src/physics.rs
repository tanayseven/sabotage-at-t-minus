//! Global physics configuration.

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::config::GRAVITY;

/// Overrides Rapier's default gravity with the value this game is tuned for.
pub fn configure_physics(mut configs: Query<&mut RapierConfiguration>) {
    for mut config in &mut configs {
        config.gravity = Vec2::new(0.0, -GRAVITY);
    }
}
