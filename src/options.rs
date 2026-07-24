use bevy::prelude::*;

use crate::settings::{Settings, VOLUME_STEP, VolumeChannel};
use crate::ui::{ACCENT, BACKDROP, BackButton, MUTED_TEXT, spawn_back_button, spawn_button};

const TRACK_WIDTH: f32 = 320.0;
const TRACK_HEIGHT: f32 = 14.0;
const LABEL_WIDTH: f32 = 110.0;
const READOUT_WIDTH: f32 = 64.0;

const STEP_BUTTON_SIZE: Vec2 = Vec2::new(48.0, 48.0);
const STEP_BUTTON_FONT: f32 = 26.0;

const TRACK_BG: Color = Color::srgb(0.16, 0.18, 0.23);

#[derive(Component)]
pub struct OptionsScreen;

/// A `-` or `+` next to one channel; the sign lives in `delta`.
#[derive(Component, Clone, Copy)]
pub struct VolumeStep {
    channel: VolumeChannel,
    delta: f32,
}

/// The filled portion of a channel's bar. Its width is the level.
#[derive(Component, Clone, Copy)]
pub struct VolumeFill(VolumeChannel);

/// The `40%` readout next to a channel's bar.
#[derive(Component, Clone, Copy)]
pub struct VolumeReadout(VolumeChannel);

pub fn spawn_options(mut commands: Commands, settings: Res<Settings>) {
    commands
        .spawn((
            OptionsScreen,
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: px(28),
                ..default()
            },
            BackgroundColor(BACKDROP),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Options"),
                TextFont {
                    font_size: FontSize::Px(56.0),
                    ..default()
                },
                TextColor(ACCENT),
                Node {
                    margin: UiRect::bottom(px(12)),
                    ..default()
                },
            ));

            for channel in VolumeChannel::ALL {
                spawn_volume_row(parent, channel, channel.get(&settings));
            }

            spawn_back_button(parent);
        });
}

fn spawn_volume_row(parent: &mut ChildSpawnerCommands, channel: VolumeChannel, level: f32) {
    parent
        .spawn(Node {
            align_items: AlignItems::Center,
            column_gap: px(16),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new(channel.label()),
                TextFont {
                    font_size: FontSize::Px(26.0),
                    ..default()
                },
                TextColor(MUTED_TEXT),
                Node {
                    width: px(LABEL_WIDTH),
                    ..default()
                },
            ));

            spawn_button(
                row,
                "-",
                VolumeStep {
                    channel,
                    delta: -VOLUME_STEP,
                },
                STEP_BUTTON_SIZE,
                STEP_BUTTON_FONT,
            );

            row.spawn((
                Node {
                    width: px(TRACK_WIDTH),
                    height: px(TRACK_HEIGHT),
                    border_radius: BorderRadius::all(px(999)),
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(TRACK_BG),
            ))
            .with_children(|track| {
                track.spawn((
                    VolumeFill(channel),
                    Node {
                        width: percent(level * 100.0),
                        height: percent(100),
                        border_radius: BorderRadius::all(px(999)),
                        ..default()
                    },
                    BackgroundColor(ACCENT),
                ));
            });

            spawn_button(
                row,
                "+",
                VolumeStep {
                    channel,
                    delta: VOLUME_STEP,
                },
                STEP_BUTTON_SIZE,
                STEP_BUTTON_FONT,
            );

            row.spawn((
                VolumeReadout(channel),
                Text::new(percent_label(level)),
                TextFont {
                    font_size: FontSize::Px(24.0),
                    ..default()
                },
                TextColor(MUTED_TEXT),
                Node {
                    width: px(READOUT_WIDTH),
                    ..default()
                },
            ));
        });
}

fn percent_label(level: f32) -> String {
    format!("{}%", (level * 100.0).round())
}

pub fn volume_step_action(
    buttons: Query<(&Interaction, &VolumeStep), Changed<Interaction>>,
    mut settings: ResMut<Settings>,
) {
    for (interaction, step) in &buttons {
        if *interaction == Interaction::Pressed {
            step.channel.adjust(&mut settings, step.delta);
        }
    }
}

/// Redraws the bars and readouts. Driven off `Settings` rather than the button
/// presses so it also picks up a level changed from anywhere else.
pub fn sync_volume_widgets(
    settings: Res<Settings>,
    mut fills: Query<(&VolumeFill, &mut Node)>,
    mut readouts: Query<(&VolumeReadout, &mut Text)>,
) {
    if !settings.is_changed() {
        return;
    }

    for (fill, mut node) in &mut fills {
        node.width = percent(fill.0.get(&settings) * 100.0);
    }
    for (readout, mut text) in &mut readouts {
        text.0 = percent_label(readout.0.get(&settings));
    }
}

pub fn despawn_options(mut commands: Commands, screen: Query<Entity, With<OptionsScreen>>) {
    for entity in &screen {
        commands.entity(entity).despawn();
    }
}

/// Shared by the options and credits screens: either button or `Escape` returns
/// to the main menu.
pub fn back_to_menu(
    buttons: Query<&Interaction, (Changed<Interaction>, With<BackButton>)>,
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<crate::state::GameState>>,
) {
    let clicked = buttons
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed);

    if clicked || keys.any_just_pressed([KeyCode::Escape, KeyCode::Backspace]) {
        next_state.set(crate::state::GameState::Menu);
    }
}
