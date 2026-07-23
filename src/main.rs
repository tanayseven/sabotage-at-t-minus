//! Sabotage at T-Minus — entry point.
//!
//! Right now this is a scaffold: a window, a camera, and a square you can
//! drive around, so that every part of the build/ship pipeline has something
//! real to carry. Replace `game` with the actual game.

use bevy::prelude::*;

const PLAYER_SPEED: f32 = 400.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Sabotage at T-Minus".into(),
                canvas: Some("#game-canvas".into()),
                fit_canvas_to_parent: true,
                // Let the browser keep handling F5, ctrl+L and friends.
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, move_player)
        .run();
}

#[derive(Component)]
struct Player;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands.spawn((
        Player,
        Sprite {
            color: Color::srgb(0.9, 0.35, 0.2),
            custom_size: Some(Vec2::splat(64.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    commands.spawn((
        Text::new("Sabotage at T-Minus\nWASD / arrows to move"),
        Node {
            position_type: PositionType::Absolute,
            top: px(16),
            left: px(16),
            ..default()
        },
    ));
}

fn move_player(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut players: Query<&mut Transform, With<Player>>,
) {
    let mut direction = Vec2::ZERO;
    if keys.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]) {
        direction.y += 1.0;
    }
    if keys.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]) {
        direction.y -= 1.0;
    }
    if keys.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]) {
        direction.x -= 1.0;
    }
    if keys.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]) {
        direction.x += 1.0;
    }

    let Some(direction) = direction.try_normalize() else {
        return;
    };

    for mut transform in &mut players {
        transform.translation += (direction * PLAYER_SPEED * time.delta_secs()).extend(0.0);
    }
}
