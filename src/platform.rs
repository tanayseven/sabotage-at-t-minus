use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::config::PLATFORM_HEIGHT;
use crate::setup::GameEntity;
use crate::tiles::{Axis, TileRun, TileSet};

const PLATFORM_Z: f32 = -1.0;

struct Platform {
    centre: Vec2,
    width: f32,
}

impl Platform {
    fn spawn(&self, commands: &mut Commands, tiles: &TileSet) {
        commands.spawn((
            GameEntity,
            Transform::from_xyz(self.centre.x, self.centre.y, PLATFORM_Z),
            RigidBody::Fixed,
            Collider::cuboid(self.width / 2.0, PLATFORM_HEIGHT / 2.0),
        ));

        TileRun {
            centre: self.centre,
            axis: Axis::Horizontal,
            length: self.width,
            thickness: PLATFORM_HEIGHT,
            z: PLATFORM_Z,
        }
        .spawn(commands, tiles);
    }
}

pub fn spawn_platforms(commands: &mut Commands, tiles: &TileSet) {
    let platforms = [
        Platform {
            centre: Vec2::new(-380.0, -120.0),
            width: 360.0,
        },
        Platform {
            centre: Vec2::new(40.0, 60.0),
            width: 300.0,
        },
        Platform {
            centre: Vec2::new(430.0, -40.0),
            width: 260.0,
        },
    ];

    for platform in platforms {
        platform.spawn(commands, tiles);
    }
}
