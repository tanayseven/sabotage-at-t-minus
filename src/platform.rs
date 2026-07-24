use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::config::PLATFORM_HEIGHT;
use crate::setup::GameEntity;

struct Platform {
    centre: Vec2,
    width: f32,
}

impl Platform {
    fn spawn(&self, commands: &mut Commands) {
        commands.spawn((
            GameEntity,
            Sprite {
                color: Color::srgb(0.26, 0.30, 0.38),
                custom_size: Some(Vec2::new(self.width, PLATFORM_HEIGHT)),
                ..default()
            },
            Transform::from_xyz(self.centre.x, self.centre.y, -1.0),
            RigidBody::Fixed,
            Collider::cuboid(self.width / 2.0, PLATFORM_HEIGHT / 2.0),
        ));
    }
}

pub fn spawn_platforms(commands: &mut Commands) {
    let platforms = [
        Platform {
            centre: Vec2::new(-380.0, -120.0),
            width: 360.0,
        },
        Platform {
            centre: Vec2::new(40.0, 60.0),
            width: 300.0,
        },
        Platform {
            centre: Vec2::new(430.0, -40.0),
            width: 260.0,
        },
    ];

    for platform in platforms {
        platform.spawn(commands);
    }
}
