//! The difficulty screen: picks how many decks — and so how many rooms — the
//! next run deals. Reached from the menu's "Play" button, and the last stop
//! before the launch pad.

use bevy::prelude::*;
use bevy::text::LineBreak;

use crate::settings::Settings;
use crate::state::GameState;
use crate::ui::{ACCENT, BACKDROP, GameFont, MUTED_TEXT, spawn_button};

const STEP_BUTTON_SIZE: Vec2 = Vec2::new(48.0, 48.0);
const STEP_BUTTON_FONT: f32 = 26.0;
const READOUT_WIDTH: f32 = 220.0;
const READOUT_FONT: f32 = 30.0;

const ACTION_BUTTON_SIZE: Vec2 = Vec2::new(200.0, 56.0);
const ACTION_BUTTON_FONT: f32 = 24.0;

#[derive(Component)]
pub struct DifficultyScreen;

/// A `-` or `+` button; the sign lives in the field.
#[derive(Component, Clone, Copy)]
pub struct DifficultyStep(isize);

/// The tier's name, between the two step buttons.
#[derive(Component)]
pub struct DifficultyReadout;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum DifficultyAction {
    Back,
    Start,
}

pub fn spawn_difficulty_screen(
    mut commands: Commands,
    settings: Res<Settings>,
    font: Res<GameFont>,
) {
    commands
        .spawn((
            DifficultyScreen,
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
                Text::new("Choose Difficulty"),
                font.at(48.0),
                TextColor(ACCENT),
                Node {
                    margin: UiRect::bottom(px(12)),
                    ..default()
                },
            ));

            parent
                .spawn(Node {
                    align_items: AlignItems::Center,
                    column_gap: px(16),
                    ..default()
                })
                .with_children(|row| {
                    spawn_button(
                        row,
                        "-",
                        DifficultyStep(-1),
                        STEP_BUTTON_SIZE,
                        STEP_BUTTON_FONT,
                    );

                    row.spawn((
                        DifficultyReadout,
                        Text::new(settings.difficulty.label()),
                        TextFont {
                            font_size: FontSize::Px(READOUT_FONT),
                            ..default()
                        },
                        TextLayout::new(Justify::Center, LineBreak::NoWrap),
                        TextColor(MUTED_TEXT),
                        Node {
                            width: px(READOUT_WIDTH),
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                    ));

                    spawn_button(
                        row,
                        "+",
                        DifficultyStep(1),
                        STEP_BUTTON_SIZE,
                        STEP_BUTTON_FONT,
                    );
                });

            parent
                .spawn(Node {
                    column_gap: px(20),
                    margin: UiRect::top(px(8)),
                    ..default()
                })
                .with_children(|row| {
                    spawn_button(
                        row,
                        "Back",
                        DifficultyAction::Back,
                        ACTION_BUTTON_SIZE,
                        ACTION_BUTTON_FONT,
                    );
                    spawn_button(
                        row,
                        "Start",
                        DifficultyAction::Start,
                        ACTION_BUTTON_SIZE,
                        ACTION_BUTTON_FONT,
                    );
                });
        });
}

pub fn difficulty_step_action(
    buttons: Query<(&Interaction, &DifficultyStep), Changed<Interaction>>,
    mut settings: ResMut<Settings>,
) {
    for (interaction, step) in &buttons {
        if *interaction == Interaction::Pressed {
            settings.difficulty = settings.difficulty.step(step.0);
        }
    }
}

/// Redraws the readout. Driven off `Settings` rather than the button presses
/// so it also picks up a tier changed from anywhere else.
pub fn sync_difficulty_readout(
    settings: Res<Settings>,
    mut readouts: Query<&mut Text, With<DifficultyReadout>>,
) {
    if !settings.is_changed() {
        return;
    }

    for mut text in &mut readouts {
        text.0 = settings.difficulty.label().to_string();
    }
}

pub fn difficulty_screen_action(
    buttons: Query<(&Interaction, &DifficultyAction), Changed<Interaction>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let mut chosen = None;

    for (interaction, action) in &buttons {
        if *interaction == Interaction::Pressed {
            chosen = Some(*action);
        }
    }

    if keys.any_just_pressed([KeyCode::Enter, KeyCode::Space]) {
        chosen = Some(DifficultyAction::Start);
    }
    if keys.just_pressed(KeyCode::Escape) {
        chosen = Some(DifficultyAction::Back);
    }

    match chosen {
        Some(DifficultyAction::Start) => next_state.set(GameState::Launchpad),
        Some(DifficultyAction::Back) => next_state.set(GameState::Menu),
        None => {}
    }
}

pub fn despawn_difficulty_screen(
    mut commands: Commands,
    screen: Query<Entity, With<DifficultyScreen>>,
) {
    for entity in &screen {
        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::{
        DifficultyAction, DifficultyReadout, DifficultyStep, difficulty_screen_action,
        difficulty_step_action, sync_difficulty_readout,
    };
    use crate::difficulty::Difficulty;
    use crate::settings::Settings;
    use crate::state::GameState;

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins(bevy::state::app::StatesPlugin)
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<Settings>()
            .insert_state(GameState::Difficulty)
            .add_systems(
                Update,
                (
                    difficulty_step_action,
                    sync_difficulty_readout,
                    difficulty_screen_action,
                )
                    .chain(),
            );
        app
    }

    fn press(app: &mut App, entity: Entity) {
        app.world_mut()
            .entity_mut(entity)
            .insert(Interaction::Pressed);
        app.update();
        app.world_mut().entity_mut(entity).insert(Interaction::None);
    }

    #[test]
    fn stepping_up_moves_settings_and_the_readout() {
        let mut app = app();
        let step_up = app.world_mut().spawn(DifficultyStep(1)).id();
        let readout = app
            .world_mut()
            .spawn((DifficultyReadout, Text::new(Difficulty::Medium.label())))
            .id();

        press(&mut app, step_up);

        assert_eq!(
            app.world().resource::<Settings>().difficulty,
            Difficulty::Hard
        );
        assert_eq!(
            app.world().entity(readout).get::<Text>().unwrap().0,
            Difficulty::Hard.label()
        );
    }

    #[test]
    fn start_moves_to_the_launchpad() {
        let mut app = app();
        let start = app.world_mut().spawn(DifficultyAction::Start).id();

        press(&mut app, start);
        app.update();

        assert_eq!(
            *app.world().resource::<State<GameState>>().get(),
            GameState::Launchpad
        );
    }

    #[test]
    fn back_returns_to_the_menu() {
        let mut app = app();
        let back = app.world_mut().spawn(DifficultyAction::Back).id();

        press(&mut app, back);
        app.update();

        assert_eq!(
            *app.world().resource::<State<GameState>>().get(),
            GameState::Menu
        );
    }
}
