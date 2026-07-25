use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

const SMALLEST_CRATE_SIZE: f32 = 44.0;
const CRATE_SIZE_STEP: f32 = 8.0;

pub fn spawn_props(commands: &mut Commands, positions: &[Vec2], marker: impl Bundle + Clone) {
    for (index, position) in positions.iter().enumerate() {
        let size = SMALLEST_CRATE_SIZE + (index as f32 * CRATE_SIZE_STEP);

        commands.spawn((
            marker.clone(),
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
