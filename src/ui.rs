use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::config::{VIEW_HEIGHT, VIEW_WIDTH};
use crate::countdown::spawn_countdown;
use crate::level::{Level, LevelProgress, RoomCodes};
use crate::manual::ManualButton;
use crate::panel::{Panel, spawn_panel_status};
use crate::quit::QuitButton;
use crate::setup::GameEntity;

/// Jersey 10, the game's one typeface — the same handle [`crate::font::FontPlugin`]
/// installs as Bevy's default, wrapped here so a caller can pick a size for it.
#[derive(Resource, Clone)]
pub struct GameFont(pub Handle<Font>);

impl GameFont {
    /// The one way this game builds a [`TextFont`]; the size is all a caller
    /// ever varies.
    pub fn at(&self, font_size: f32) -> TextFont {
        TextFont::from_font_size(FontSize::Px(font_size)).with_font(self.0.clone())
    }
}

pub const ACCENT: Color = Color::srgb(0.9, 0.35, 0.2);
/// The two colours the title screen letters the game's name in. Shared with the
/// in-level HUD, so a run reads as the same game the menu introduced.
pub const TITLE_BLUE: Color = Color::srgb(0.31, 0.48, 0.86);
pub const TITLE_CRIMSON: Color = Color::srgb(0.79, 0.11, 0.24);
/// The mission clock, right up until it goes hot and turns [`ACCENT`].
pub const CLOCK_GREEN: Color = Color::srgb(0.55, 0.90, 0.62);
pub const MUTED_TEXT: Color = Color::srgb(0.75, 0.77, 0.82);
/// Shared by every full-screen menu page, so they cut between each other
/// without a flash of a different colour.
pub const BACKDROP: Color = Color::srgb(0.08, 0.09, 0.12);

const NORMAL_BUTTON: Color = Color::srgb(0.20, 0.22, 0.28);
const HOVERED_BUTTON: Color = Color::srgb(0.28, 0.32, 0.40);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.45, 0.60);

const HUD_BUTTON_SIZE: Vec2 = Vec2::new(120.0, 44.0);
const HUD_BUTTON_FONT: f32 = 22.0;
const HUD_BUTTON_GAP: f32 = 12.0;
const HUD_COUNTDOWN_FONT: f32 = 56.0;
/// The level's name, in the corner. A little under the launch pad's, which has
/// a whole empty screen to itself rather than a run going on underneath — but
/// not much under: the pixel face stops reading below the mid-thirties.
const HUD_TITLE_FONT: f32 = 36.0;
const HUD_STATUS_FONT: f32 = 20.0;
/// 1.5x the engine's default 20px, so the controls read at a glance rather
/// than needing to be sought out.
const HUD_CONTROLS_FONT: f32 = 30.0;

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
    spawn_labelled_button(parent, label, action, size, font_size, Color::WHITE);
}

/// As [`spawn_button`], but for the title screen, whose labels are tinted to
/// match its two-tone heading.
pub fn spawn_labelled_button(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    action: impl Component,
    size: Vec2,
    font_size: f32,
    label_color: Color,
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
                TextColor(label_color),
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

pub fn spawn_hud(
    commands: &mut Commands,
    font: &GameFont,
    codes: &RoomCodes,
    level: Level,
    panel: &Panel,
    progress: &LevelProgress,
) {
    // The level names itself in the corner, the way the launch pad does.
    commands.spawn((
        GameEntity,
        Text::new(level.title()),
        font.at(HUD_TITLE_FONT),
        TextColor(TITLE_BLUE),
        Node {
            position_type: PositionType::Absolute,
            top: px(16),
            left: px(16),
            ..default()
        },
    ));

    // Along the bottom rather than under the title: the top edge is spoken for
    // by the clock and the panel status, both of them centred, and the controls
    // are long enough lines to run into them.
    commands.spawn((
        GameEntity,
        Text::new(
            "A/D to move, W / space to jump, W/S on a ladder\nE to work a door or throw a switch — M for the repair manual",
        ),
        TextFont {
            font_size: FontSize::Px(HUD_CONTROLS_FONT),
            ..default()
        },
        TextColor(ACCENT),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(16),
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
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|parent| {
            spawn_countdown(parent, HUD_COUNTDOWN_FONT);
            // Under the clock, because it is read the same way: a glance, not a
            // look. It names the room the panel is in — finding it is not the
            // puzzle, and the clock is short.
            spawn_panel_status(parent, codes, level, panel, progress, HUD_STATUS_FONT);
        });

    commands
        .spawn((
            GameEntity,
            Node {
                position_type: PositionType::Absolute,
                top: px(16),
                right: px(16),
                column_gap: px(HUD_BUTTON_GAP),
                ..default()
            },
        ))
        .with_children(|parent| {
            spawn_button(
                parent,
                "Manual",
                ManualButton,
                HUD_BUTTON_SIZE,
                HUD_BUTTON_FONT,
            );
            spawn_button(parent, "Quit", QuitButton, HUD_BUTTON_SIZE, HUD_BUTTON_FONT);
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
