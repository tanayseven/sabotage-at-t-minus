use bevy::prelude::*;

use crate::level::{Level, PendingLevelAdvance};
use crate::state::{GameState, PlayingState};
use crate::ui::{ACCENT, MUTED_TEXT, spawn_button};

const SCRIM: Color = Color::srgba(0.0, 0.0, 0.0, 0.72);
const PANEL: Color = Color::srgb(0.10, 0.16, 0.11);

const DIALOG_BUTTON_SIZE: Vec2 = Vec2::new(220.0, 56.0);
const DIALOG_BUTTON_FONT: f32 = 22.0;

#[derive(Component)]
pub struct MissionCompleteScreen;

#[derive(Component)]
pub struct MissionCompleteButton;

pub fn spawn_mission_complete(mut commands: Commands, level: Res<Level>) {
    let button_label = if level.next().is_some() {
        "Next Level"
    } else {
        "Back to Menu"
    };

    commands
        .spawn((
            MissionCompleteScreen,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(SCRIM),
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
                        Text::new("Mission Complete"),
                        TextFont {
                            font_size: FontSize::Px(40.0),
                            ..default()
                        },
                        TextColor(ACCENT),
                    ));
                    panel.spawn((
                        Text::new("Portal challenge cleared. Ready for the next stage."),
                        TextFont {
                            font_size: FontSize::Px(22.0),
                            ..default()
                        },
                        TextColor(MUTED_TEXT),
                    ));

                    panel
                        .spawn(Node {
                            margin: UiRect::top(px(12)),
                            ..default()
                        })
                        .with_children(|row| {
                            spawn_button(row, button_label, MissionCompleteButton, DIALOG_BUTTON_SIZE, DIALOG_BUTTON_FONT);
                        });
                });
        });
}

#[allow(clippy::type_complexity)]
pub fn mission_complete_action(
    buttons: Query<&Interaction, (Changed<Interaction>, With<MissionCompleteButton>)>,
    level: Res<Level>,
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut next_playing: ResMut<NextState<PlayingState>>,
    mut next_game: ResMut<NextState<GameState>>,
) {
    let clicked = buttons
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed);

    if clicked || keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Escape) {
        if let Some(next_level) = level.next() {
            commands.insert_resource(PendingLevelAdvance(next_level));
            next_playing.set(PlayingState::Running);
        } else {
            next_game.set(GameState::Menu);
        }
    }
}

pub fn despawn_mission_complete(
    mut commands: Commands,
    screens: Query<Entity, With<MissionCompleteScreen>>,
) {
    for entity in &screens {
        commands.entity(entity).despawn();
    }
}
