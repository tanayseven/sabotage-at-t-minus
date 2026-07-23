//! The static walls that box in the design area.

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::config::{DESIGN_HEIGHT, DESIGN_WIDTH, WALL_THICKNESS};

/// A single static wall: its centre and half-extents in world units.
struct Wall {
    centre: Vec2,
    half_extents: Vec2,
}

impl Wall {
    fn spawn(&self, commands: &mut Commands) {
        commands.spawn((
            Sprite {
                color: Color::srgb(0.18, 0.20, 0.26),
                custom_size: Some(self.half_extents * 2.0),
                ..default()
            },
            Transform::from_xyz(self.centre.x, self.centre.y, -5.0),
            RigidBody::Fixed,
            Collider::cuboid(self.half_extents.x, self.half_extents.y),
        ));
    }
}

/// Static colliders around the design area. The bottom one doubles as the
/// ground; the rest stop anything leaving the region the camera guarantees is
/// visible.
///
/// Each wall sits just outside the design area so its inner face lines up with
/// the edge.
pub fn spawn_walls(commands: &mut Commands) {
    let half_width = DESIGN_WIDTH / 2.0;
    let half_height = DESIGN_HEIGHT / 2.0;
    let half_thickness = WALL_THICKNESS / 2.0;

    let walls = [
        // Top
        Wall {
            centre: Vec2::new(0.0, half_height + half_thickness),
            half_extents: Vec2::new(half_width + WALL_THICKNESS, half_thickness),
        },
        // Bottom (the ground)
        Wall {
            centre: Vec2::new(0.0, -half_height - half_thickness),
            half_extents: Vec2::new(half_width + WALL_THICKNESS, half_thickness),
        },
        // Left
        Wall {
            centre: Vec2::new(-half_width - half_thickness, 0.0),
            half_extents: Vec2::new(half_thickness, half_height + WALL_THICKNESS),
        },
        // Right
        Wall {
            centre: Vec2::new(half_width + half_thickness, 0.0),
            half_extents: Vec2::new(half_thickness, half_height + WALL_THICKNESS),
        },
    ];

    for wall in walls {
        wall.spawn(commands);
    }
}
