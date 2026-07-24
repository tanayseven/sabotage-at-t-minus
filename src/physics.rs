use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::config::GRAVITY;

pub fn configure_physics(mut configs: Query<&mut RapierConfiguration>) {
    for mut config in &mut configs {
        config.gravity = Vec2::new(0.0, -GRAVITY);
    }
}

pub fn pause_physics(mut configs: Query<&mut RapierConfiguration>) {
    set_physics_active(&mut configs, false);
}

pub fn resume_physics(mut configs: Query<&mut RapierConfiguration>) {
    set_physics_active(&mut configs, true);
}

fn set_physics_active(configs: &mut Query<&mut RapierConfiguration>, active: bool) {
    for mut config in configs {
        config.physics_pipeline_active = active;
    }
}
