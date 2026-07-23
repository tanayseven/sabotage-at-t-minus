//! The ledges the player lands on and jumps between.

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::config::PLATFORM_HEIGHT;

/// Spawns the ledges. Vertical gaps are kept under the jump apex so every
/// platform is reachable from the one before it.
pub fn spawn_platforms(commands: &mut Commands) {
    let platforms = [
        (Vec2::new(-380.0, -120.0), 360.0),
        (Vec2::new(40.0, 60.0), 300.0),
        (Vec2::new(430.0, -40.0), 260.0),
    ];

    for (centre, width) in platforms {
        commands.spawn((
            Sprite {
                color: Color::srgb(0.26, 0.30, 0.38),
                custom_size: Some(Vec2::new(width, PLATFORM_HEIGHT)),
                ..default()
            },
            Transform::from_xyz(centre.x, centre.y, -1.0),
            RigidBody::Fixed,
            Collider::cuboid(width / 2.0, PLATFORM_HEIGHT / 2.0),
        ));
    }
}
