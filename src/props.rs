//! Loose dynamic boxes, there to make the physics visible at a glance.

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

/// Spawns a handful of crates to shove off the ledges.
pub fn spawn_props(commands: &mut Commands) {
    for (index, position) in [
        Vec2::new(-300.0, 40.0),
        Vec2::new(120.0, 300.0),
        Vec2::new(200.0, 300.0),
        Vec2::new(430.0, 120.0),
    ]
    .into_iter()
    .enumerate()
    {
        let size = 44.0 + (index as f32 * 8.0);
        commands.spawn((
            Sprite {
                color: Color::srgb(0.35, 0.45, 0.6),
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
            Transform::from_xyz(position.x, position.y, 0.0),
            RigidBody::Dynamic,
            Collider::cuboid(size / 2.0, size / 2.0),
            Restitution::coefficient(0.1),
        ));
    }
}
