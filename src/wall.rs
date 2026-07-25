use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::config::{DESIGN_HEIGHT, DESIGN_WIDTH, WALL_THICKNESS};
use crate::tiles::{Axis, TileRun, TileSet};

const WALL_Z: f32 = -5.0;

struct Wall {
    centre: Vec2,
    axis: Axis,
    length: f32,
}

impl Wall {
    fn spawn(&self, commands: &mut Commands, tiles: &TileSet, marker: impl Bundle + Clone) {
        let half_extents = self.axis.half_extents(self.length, WALL_THICKNESS);

        commands.spawn((
            marker.clone(),
            Transform::from_xyz(self.centre.x, self.centre.y, WALL_Z),
            RigidBody::Fixed,
            Collider::cuboid(half_extents.x, half_extents.y),
        ));

        TileRun {
            centre: self.centre,
            axis: self.axis,
            length: self.length,
            thickness: WALL_THICKNESS,
            z: WALL_Z,
        }
        .spawn(commands, tiles, marker);
    }
}

/// The box that frames the play area. Both the launch pad and the level proper
/// are fought inside it, so the caller says which screen owns the walls.
pub fn spawn_walls(commands: &mut Commands, tiles: &TileSet, marker: impl Bundle + Clone) {
    let half_width = DESIGN_WIDTH / 2.0;
    let half_height = DESIGN_HEIGHT / 2.0;
    let half_thickness = WALL_THICKNESS / 2.0;

    let horizontal_length = DESIGN_WIDTH + WALL_THICKNESS * 2.0;
    let vertical_length = DESIGN_HEIGHT + WALL_THICKNESS * 2.0;

    let top = Wall {
        centre: Vec2::new(0.0, half_height + half_thickness),
        axis: Axis::Horizontal,
        length: horizontal_length,
    };
    let ground = Wall {
        centre: Vec2::new(0.0, -half_height - half_thickness),
        axis: Axis::Horizontal,
        length: horizontal_length,
    };
    let left = Wall {
        centre: Vec2::new(-half_width - half_thickness, 0.0),
        axis: Axis::Vertical,
        length: vertical_length,
    };
    let right = Wall {
        centre: Vec2::new(half_width + half_thickness, 0.0),
        axis: Axis::Vertical,
        length: vertical_length,
    };

    for wall in [top, ground, left, right] {
        wall.spawn(commands, tiles, marker.clone());
    }
}
