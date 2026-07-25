use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::config::PLATFORM_HEIGHT;
use crate::tiles::{Axis, TileRun, TileSet};

const PLATFORM_Z: f32 = -1.0;

/// A solid ledge you can stand on. The launch pad's stairway and gantry bridge
/// are made of these too, which is why it is public.
#[derive(Debug)]
pub struct Platform {
    pub centre: Vec2,
    pub width: f32,
}

impl Platform {
    pub const fn new(x: f32, y: f32, width: f32) -> Self {
        Self {
            centre: Vec2::new(x, y),
            width,
        }
    }

    /// Built from the surface the player lands on, which is what a level layout
    /// is actually reasoned about, rather than the centre the collider needs.
    pub const fn with_top(x: f32, top: f32, width: f32) -> Self {
        Self::new(x, top - PLATFORM_HEIGHT / 2.0, width)
    }

    pub fn spawn(&self, commands: &mut Commands, tiles: &TileSet, marker: impl Bundle + Clone) {
        commands.spawn((
            marker.clone(),
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
        .spawn(commands, tiles, marker);
    }
}

pub fn spawn_platforms(
    commands: &mut Commands,
    tiles: &TileSet,
    platforms: &[Platform],
    marker: impl Bundle + Clone,
) {
    for platform in platforms {
        platform.spawn(commands, tiles, marker.clone());
    }
}
