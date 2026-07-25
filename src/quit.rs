use bevy::prelude::*;

use crate::state::{GameState, PlayingState};
use crate::ui::{ACCENT, MUTED_TEXT, spawn_button};

const SCRIM: Color = Color::srgba(0.0, 0.0, 0.0, 0.72);
const PANEL: Color = Color::srgb(0.12, 0.13, 0.17);

const DIALOG_BUTTON_SIZE: Vec2 = Vec2::new(200.0, 56.0);
const DIALOG_BUTTON_FONT: f32 = 22.0;

/// The HUD button that opens the dialog.
#[derive(Component)]
pub struct QuitButton;

#[derive(Component)]
pub struct QuitDialog;

/// Which gameplay sub-state to return to when dismissing the quit dialog.
#[derive(Resource, Debug, Clone)]
pub struct QuitResumeState(pub PlayingState);

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum QuitChoice {
    ToMenu,
    KeepPlaying,
}

#[allow(clippy::type_complexity)]
pub fn open_quit_dialog(
    mut commands: Commands,
    buttons: Query<&Interaction, (Changed<Interaction>, With<QuitButton>)>,
    keys: Res<ButtonInput<KeyCode>>,
    playing: Res<State<PlayingState>>,
    mut next_playing: ResMut<NextState<PlayingState>>,
) {
    let clicked = buttons
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed);

    if clicked || keys.just_pressed(KeyCode::Escape) {
        commands.insert_resource(QuitResumeState(playing.get().clone()));
        next_playing.set(PlayingState::ConfirmQuit);
    }
}

pub fn spawn_quit_dialog(mut commands: Commands) {
    commands
        .spawn((
            QuitDialog,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(SCRIM),
            // Above the HUD, so the dialog is unambiguously modal.
            GlobalZIndex(1),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        padding: UiRect::axes(px(56), px(40)),
                        row_gap: px(20),
                        border_radius: BorderRadius::all(px(12)),
                        ..default()
                    },
                    BackgroundColor(PANEL),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new("Abort the mission?"),
                        TextFont {
                            font_size: FontSize::Px(40.0),
                            ..default()
                        },
                        TextColor(ACCENT),
                    ));
                    panel.spawn((
                        Text::new("You'll be sent back to the main menu."),
                        TextFont {
                            font_size: FontSize::Px(22.0),
                            ..default()
                        },
                        TextColor(MUTED_TEXT),
                    ));

                    panel
                        .spawn(Node {
                            column_gap: px(16),
                            margin: UiRect::top(px(12)),
                            ..default()
                        })
                        .with_children(|row| {
                            spawn_button(
                                row,
                                "Quit to Menu",
                                QuitChoice::ToMenu,
                                DIALOG_BUTTON_SIZE,
                                DIALOG_BUTTON_FONT,
                            );
                            spawn_button(
                                row,
                                "Keep Playing",
                                QuitChoice::KeepPlaying,
                                DIALOG_BUTTON_SIZE,
                                DIALOG_BUTTON_FONT,
                            );
                        });
                });
        });
}

pub fn quit_dialog_action(
    buttons: Query<(&Interaction, &QuitChoice), Changed<Interaction>>,
    keys: Res<ButtonInput<KeyCode>>,
    resume: Option<Res<QuitResumeState>>,
    mut next_game: ResMut<NextState<GameState>>,
    mut next_playing: ResMut<NextState<PlayingState>>,
) {
    let mut chosen = None;

    for (interaction, choice) in &buttons {
        if *interaction == Interaction::Pressed {
            chosen = Some(*choice);
        }
    }

    if keys.just_pressed(KeyCode::Enter) {
        chosen = Some(QuitChoice::ToMenu);
    }
    if keys.just_pressed(KeyCode::Escape) {
        chosen = Some(QuitChoice::KeepPlaying);
    }

    match chosen {
        // Leaving `Playing` tears the sub-state down with it, so the dialog is
        // despawned by `despawn_quit_dialog` on the same transition.
        Some(QuitChoice::ToMenu) => next_game.set(GameState::Menu),
        Some(QuitChoice::KeepPlaying) => {
            let fallback = PlayingState::Running;
            next_playing.set(resume.map_or(fallback, |state| state.0.clone()));
        }
        None => {}
    }
}

pub fn despawn_quit_dialog(mut commands: Commands, dialog: Query<Entity, With<QuitDialog>>) {
    for entity in &dialog {
        commands.entity(entity).despawn();
    }

    commands.remove_resource::<QuitResumeState>();
}
