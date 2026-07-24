use bevy::prelude::*;

use crate::state::GameState;
use crate::ui::{ACCENT, MUTED_TEXT, spawn_button};

const SCRIM: Color = Color::srgba(0.0, 0.0, 0.0, 0.72);
const PANEL: Color = Color::srgb(0.12, 0.13, 0.17);

const DIALOG_BUTTON_SIZE: Vec2 = Vec2::new(200.0, 56.0);
const DIALOG_BUTTON_FONT: f32 = 22.0;

#[derive(Component)]
pub struct GameOverScreen;

#[derive(Component)]
pub struct GameOverButton;

pub fn spawn_game_over(mut commands: Commands) {
    commands
        .spawn((
            GameOverScreen,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(SCRIM),
            // Above the HUD, matching the confirm-quit dialog.
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
                        Text::new("Game Over"),
                        TextFont {
                            font_size: FontSize::Px(40.0),
                            ..default()
                        },
                        TextColor(ACCENT),
                    ));
                    panel.spawn((
                        Text::new("The clock ran out. The rocket launched without you."),
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
                            spawn_button(
                                row,
                                "Back to Menu",
                                GameOverButton,
                                DIALOG_BUTTON_SIZE,
                                DIALOG_BUTTON_FONT,
                            );
                        });
                });
        });
}

#[allow(clippy::type_complexity)]
pub fn game_over_action(
    buttons: Query<&Interaction, (Changed<Interaction>, With<GameOverButton>)>,
    keys: Res<ButtonInput<KeyCode>>,
    mut next_game: ResMut<NextState<GameState>>,
) {
    let clicked = buttons
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed);

    if clicked || keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Escape) {
        next_game.set(GameState::Menu);
    }
}

pub fn despawn_game_over(mut commands: Commands, screens: Query<Entity, With<GameOverScreen>>) {
    for entity in &screens {
        commands.entity(entity).despawn();
    }
}
