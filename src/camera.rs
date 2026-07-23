//! The 2D camera and the backdrop that marks the guaranteed-visible area.

use bevy::camera::ScalingMode;
use bevy::prelude::*;

use crate::config::{DESIGN_HEIGHT, DESIGN_WIDTH};

/// Spawns the camera and the backdrop sprite.
pub fn spawn_camera(commands: &mut Commands) {
    commands.spawn((
        Camera2d,
        // `AutoMin` keeps the aspect ratio and guarantees the whole design area
        // stays on screen: as the window grows, the world grows with it instead
        // of simply revealing more of it. A window with a different aspect ratio
        // than the design one shows extra world on the roomier axis rather than
        // letterboxing, so keep anything important away from the very edges.
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: DESIGN_WIDTH,
                min_height: DESIGN_HEIGHT,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));

    // Backdrop, sized to the design area so it doubles as a visible marker of
    // what is guaranteed to be on screen at any window size.
    commands.spawn((
        Sprite {
            color: Color::srgb(0.10, 0.11, 0.14),
            custom_size: Some(Vec2::new(DESIGN_WIDTH, DESIGN_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -10.0),
    ));
}
