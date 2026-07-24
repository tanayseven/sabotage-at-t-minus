use bevy::camera::ScalingMode;
use bevy::prelude::*;

use crate::config::{DESIGN_HEIGHT, DESIGN_WIDTH, VIEW_HEIGHT, VIEW_WIDTH};

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: VIEW_WIDTH,
                min_height: VIEW_HEIGHT,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));

    commands.spawn((
        Sprite {
            color: Color::srgb(0.10, 0.11, 0.14),
            custom_size: Some(Vec2::new(DESIGN_WIDTH, DESIGN_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -10.0),
    ));
}
