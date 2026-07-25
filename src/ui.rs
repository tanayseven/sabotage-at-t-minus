use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::config::{VIEW_HEIGHT, VIEW_WIDTH};
use crate::countdown::spawn_countdown;
use crate::quit::QuitButton;
use crate::setup::GameEntity;

pub const ACCENT: Color = Color::srgb(0.9, 0.35, 0.2);
pub const MUTED_TEXT: Color = Color::srgb(0.75, 0.77, 0.82);
/// Shared by every full-screen menu page, so they cut between each other
/// without a flash of a different colour.
pub const BACKDROP: Color = Color::srgb(0.08, 0.09, 0.12);

const NORMAL_BUTTON: Color = Color::srgb(0.20, 0.22, 0.28);
const HOVERED_BUTTON: Color = Color::srgb(0.28, 0.32, 0.40);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.45, 0.60);

const HUD_QUIT_SIZE: Vec2 = Vec2::new(120.0, 44.0);
const HUD_QUIT_FONT: f32 = 22.0;
const HUD_COUNTDOWN_FONT: f32 = 56.0;

const BACK_BUTTON_SIZE: Vec2 = Vec2::new(200.0, 56.0);
const BACK_BUTTON_FONT: f32 = 24.0;

/// Returns to the main menu. Shared by the options and credits screens.
#[derive(Component)]
pub struct BackButton;

pub fn spawn_back_button(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn(Node {
            margin: UiRect::top(px(20)),
            ..default()
        })
        .with_children(|row| {
            spawn_button(row, "Back", BackButton, BACK_BUTTON_SIZE, BACK_BUTTON_FONT);
        });
}

/// Every button in the game looks the same; only the footprint and the label
/// size change between the menu and the tighter in-game controls.
pub fn spawn_button(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    action: impl Component,
    size: Vec2,
    font_size: f32,
) {
    parent
        .spawn((
            Button,
            action,
            Node {
                width: px(size.x),
                height: px(size.y),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(px(8)),
                ..default()
            },
            BackgroundColor(NORMAL_BUTTON),
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(label),
                TextFont {
                    font_size: FontSize::Px(font_size),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

#[allow(clippy::type_complexity)]
pub fn button_visuals(
    mut buttons: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, mut color) in &mut buttons {
        *color = match interaction {
            Interaction::Pressed => PRESSED_BUTTON.into(),
            Interaction::Hovered => HOVERED_BUTTON.into(),
            Interaction::None => NORMAL_BUTTON.into(),
        };
    }
}

pub fn spawn_hud(commands: &mut Commands) {
    commands.spawn((
        GameEntity,
        Text::new("Sabotage at T-Minus\nA/D to move, W / space to jump"),
        Node {
            position_type: PositionType::Absolute,
            top: px(16),
            left: px(16),
            ..default()
        },
    ));

    // The clock is the thing the player is racing, so it gets the middle of the
    // top edge to itself, at a size that reads without being looked for. The row
    // spans the full width to centre it, which would otherwise put an invisible
    // node over the quit button in the corner — hence ignoring picking.
    commands
        .spawn((
            GameEntity,
            Node {
                position_type: PositionType::Absolute,
                top: px(16),
                width: percent(100),
                justify_content: JustifyContent::Center,
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|parent| {
            spawn_countdown(parent, HUD_COUNTDOWN_FONT);
        });

    commands
        .spawn((
            GameEntity,
            Node {
                position_type: PositionType::Absolute,
                top: px(16),
                right: px(16),
                ..default()
            },
        ))
        .with_children(|parent| {
            spawn_button(parent, "Quit", QuitButton, HUD_QUIT_SIZE, HUD_QUIT_FONT);
        });
}

pub fn sync_ui_scale(
    windows: Query<&Window, (With<PrimaryWindow>, Changed<Window>)>,
    mut ui_scale: ResMut<UiScale>,
) {
    let Ok(window) = windows.single() else {
        return;
    };

    let scale = (window.width() / VIEW_WIDTH).min(window.height() / VIEW_HEIGHT);
    if !scale.is_finite() || scale <= 0.0 {
        return;
    }

    let already_correct = (ui_scale.0 - scale).abs() <= f32::EPSILON;
    if already_correct {
        return;
    }

    ui_scale.0 = scale;
}
