use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

const SMALLEST_CRATE_SIZE: f32 = 44.0;
const CRATE_SIZE_STEP: f32 = 8.0;

/// The biggest a crate is ever built, however many a level asks for. This is
/// not just trivia: a player walking a deck pushes whatever is on it along
/// ahead of them, so a crate is the thing most likely to end up between them
/// and a door. [`crate::door`] sizes its reach off this so that being stood
/// behind one is never what stops a door being worked.
pub const LARGEST_CRATE_SIZE: f32 = 80.0;

pub fn spawn_props(commands: &mut Commands, positions: &[Vec2], marker: impl Bundle + Clone) {
    for (index, position) in positions.iter().enumerate() {
        // Clamped rather than trusted to stay small, so the bound the doors are
        // built against holds however long a level's crate list grows.
        let size = (SMALLEST_CRATE_SIZE + (index as f32 * CRATE_SIZE_STEP)).min(LARGEST_CRATE_SIZE);

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
