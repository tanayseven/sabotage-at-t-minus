//! The HUD text and keeping the UI layer in step with the camera.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::config::{DESIGN_HEIGHT, DESIGN_WIDTH};

/// Spawns the on-screen instructions.
pub fn spawn_hud(commands: &mut Commands) {
    commands.spawn((
        Text::new("Sabotage at T-Minus\nA/D or arrows to move, W / space to jump"),
        Node {
            position_type: PositionType::Absolute,
            top: px(16),
            left: px(16),
            ..default()
        },
    ));
}

/// Scales the UI layer by the same factor the camera scales the world by.
///
/// Bevy's UI is laid out in window pixels and so ignores the camera projection
/// entirely; without this, text and HUD nodes would stay a fixed pixel size
/// while the sprites around them grew. Mirroring `ScalingMode::AutoMin`'s
/// factor here keeps the two layers locked together.
pub fn sync_ui_scale(
    windows: Query<&Window, (With<PrimaryWindow>, Changed<Window>)>,
    mut ui_scale: ResMut<UiScale>,
) {
    // Nothing to do on frames where the window hasn't moved or resized.
    let Ok(window) = windows.single() else {
        return;
    };

    let scale = (window.width() / DESIGN_WIDTH).min(window.height() / DESIGN_HEIGHT);
    if !scale.is_finite() || scale <= 0.0 {
        return;
    }

    // Writing through `ResMut` flags the UI for a full relayout, so only do it
    // when the factor has actually moved.
    if (ui_scale.0 - scale).abs() > f32::EPSILON {
        ui_scale.0 = scale;
    }
}
