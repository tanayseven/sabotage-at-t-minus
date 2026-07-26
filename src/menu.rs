use bevy::prelude::*;

use crate::state::GameState;
use crate::ui::{BACKDROP, GameFont, TITLE_BLUE, TITLE_CRIMSON, spawn_labelled_button};

const MENU_BUTTON_SIZE: Vec2 = Vec2::new(260.0, 64.0);
const MENU_BUTTON_FONT: f32 = 28.0;
const TITLE_FONT: f32 = 64.0;

#[derive(Component)]
pub struct MenuScreen;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum MenuButton {
    Play,
    Options,
    Credits,
    #[cfg(not(target_arch = "wasm32"))]
    Quit,
}

#[cfg(target_arch = "wasm32")]
const MENU_BUTTONS: [(&str, MenuButton); 3] = [
    ("Play", MenuButton::Play),
    ("Options", MenuButton::Options),
    ("Credits", MenuButton::Credits),
];

#[cfg(not(target_arch = "wasm32"))]
const MENU_BUTTONS: [(&str, MenuButton); 4] = [
    ("Play", MenuButton::Play),
    ("Options", MenuButton::Options),
    ("Credits", MenuButton::Credits),
    ("Quit", MenuButton::Quit),
];

pub fn spawn_menu(mut commands: Commands, font: Res<GameFont>) {
    commands
        .spawn((
            MenuScreen,
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: px(20),
                ..default()
            },
            BackgroundColor(BACKDROP),
        ))
        .with_children(|parent| {
            // Two-tone: the setup in blue, the thing being raced in crimson.
            parent
                .spawn((
                    Text::new("Sabotage at "),
                    font.at(TITLE_FONT),
                    TextColor(TITLE_BLUE),
                    Node {
                        margin: UiRect::bottom(px(16)),
                        ..default()
                    },
                ))
                .with_child((
                    TextSpan::new("T-Minus"),
                    font.at(TITLE_FONT),
                    TextColor(TITLE_CRIMSON),
                ));

            for (label, action) in MENU_BUTTONS {
                spawn_labelled_button(
                    parent,
                    label,
                    action,
                    MENU_BUTTON_SIZE,
                    MENU_BUTTON_FONT,
                    TITLE_BLUE,
                );
            }
        });
}

#[allow(clippy::type_complexity)]
pub fn menu_action(
    buttons: Query<(&Interaction, &MenuButton), (Changed<Interaction>, With<Button>)>,
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    #[cfg(not(target_arch = "wasm32"))] mut exit: MessageWriter<AppExit>,
) {
    let mut chosen = None;

    for (interaction, button) in &buttons {
        if *interaction == Interaction::Pressed {
            chosen = Some(*button);
        }
    }

    if keys.any_just_pressed([KeyCode::Enter, KeyCode::Space]) {
        chosen = Some(MenuButton::Play);
    }
    #[cfg(not(target_arch = "wasm32"))]
    if keys.just_pressed(KeyCode::Escape) {
        chosen = Some(MenuButton::Quit);
    }

    match chosen {
        Some(MenuButton::Play) => next_state.set(GameState::Difficulty),
        Some(MenuButton::Options) => next_state.set(GameState::Options),
        Some(MenuButton::Credits) => next_state.set(GameState::Credits),
        #[cfg(not(target_arch = "wasm32"))]
        Some(MenuButton::Quit) => {
            exit.write(AppExit::Success);
        }
        None => {}
    }
}

pub fn despawn_menu(mut commands: Commands, menu: Query<Entity, With<MenuScreen>>) {
    for entity in &menu {
        commands.entity(entity).despawn();
    }
}
