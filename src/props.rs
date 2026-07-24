use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::setup::GameEntity;

const SMALLEST_CRATE_SIZE: f32 = 44.0;
const CRATE_SIZE_STEP: f32 = 8.0;

const CRATE_POSITIONS: [Vec2; 4] = [
    Vec2::new(-300.0, 40.0),
    Vec2::new(120.0, 300.0),
    Vec2::new(200.0, 300.0),
    Vec2::new(430.0, 120.0),
];

pub fn spawn_props(commands: &mut Commands) {
    for (index, position) in CRATE_POSITIONS.into_iter().enumerate() {
        let size = SMALLEST_CRATE_SIZE + (index as f32 * CRATE_SIZE_STEP);

        commands.spawn((
            GameEntity,
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
