use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::config::{DESIGN_HEIGHT, DESIGN_WIDTH, WALL_THICKNESS};
use crate::tiles::{Axis, TileRun, TileSet};

const WALL_Z: f32 = -5.0;

/// A solid run of wall. The launch pad and the level proper both build theirs
/// out of these: the pad's four make a box around the viewport, the rocket's
/// make its hull and the bulkheads between its rooms.
pub struct Wall {
    pub centre: Vec2,
    pub axis: Axis,
    pub length: f32,
}

impl Wall {
    pub const fn new(centre: Vec2, axis: Axis, length: f32) -> Self {
        Self {
            centre,
            axis,
            length,
        }
    }

    /// A vertical wall spanning `bottom` to `top`. Built from its two ends
    /// rather than a centre and a length because that is how a hull and a
    /// bulkhead are actually described — this deck up to that one.
    pub const fn between(x: f32, bottom: f32, top: f32) -> Self {
        Self::new(
            Vec2::new(x, (bottom + top) / 2.0),
            Axis::Vertical,
            top - bottom,
        )
    }

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

pub fn spawn_wall_run(
    commands: &mut Commands,
    tiles: &TileSet,
    walls: &[Wall],
    marker: impl Bundle + Clone,
) {
    for wall in walls {
        wall.spawn(commands, tiles, marker.clone());
    }
}

/// The box that frames the play area. The launch pad is fought inside it, so
/// the caller says which screen owns the walls.
pub fn spawn_walls(commands: &mut Commands, tiles: &TileSet, marker: impl Bundle + Clone) {
    let half_width = DESIGN_WIDTH / 2.0;
    let half_height = DESIGN_HEIGHT / 2.0;
    let half_thickness = WALL_THICKNESS / 2.0;

    let horizontal_length = DESIGN_WIDTH + WALL_THICKNESS * 2.0;
    let vertical_length = DESIGN_HEIGHT + WALL_THICKNESS * 2.0;

    let box_walls = [
        Wall::new(
            Vec2::new(0.0, half_height + half_thickness),
            Axis::Horizontal,
            horizontal_length,
        ),
        Wall::new(
            Vec2::new(0.0, -half_height - half_thickness),
            Axis::Horizontal,
            horizontal_length,
        ),
        Wall::new(
            Vec2::new(-half_width - half_thickness, 0.0),
            Axis::Vertical,
            vertical_length,
        ),
        Wall::new(
            Vec2::new(half_width + half_thickness, 0.0),
            Axis::Vertical,
            vertical_length,
        ),
    ];

    spawn_wall_run(commands, tiles, &box_walls, marker);
}
