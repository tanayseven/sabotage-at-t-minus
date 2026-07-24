use bevy::prelude::*;
use bevy::text::LineBreak;

use crate::ui::{ACCENT, BACKDROP, MUTED_TEXT, spawn_back_button};

const ROLE_WIDTH: f32 = 320.0;
const NAME_WIDTH: f32 = 320.0;
const ROLE_FONT: f32 = 24.0;
const NAME_FONT: f32 = 28.0;
const NAME_GAP: f32 = 4.0;
const ROLE_NUDGE: f32 = (NAME_FONT - ROLE_FONT) * 0.6 / 2.0;

const CREDITS: [(&str, &[&str]); 3] = [
    ("Programming", &["Tanay PrabhuDesai"]),
    ("Art", &["John Doe"]),
    ("Music", &["Jane Doe"]),
];

const COLOPHON: [&str; 2] = [
    "Built with Bevy and Rapier",
    "Made for the GMTK Game Jam 2026",
];

#[derive(Component)]
pub struct CreditsScreen;

pub fn spawn_credits(mut commands: Commands) {
    commands
        .spawn((
            CreditsScreen,
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: px(16),
                ..default()
            },
            BackgroundColor(BACKDROP),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Sabotage at T-Minus"),
                TextFont {
                    font_size: FontSize::Px(56.0),
                    ..default()
                },
                TextColor(ACCENT),
                Node {
                    margin: UiRect::bottom(px(20)),
                    ..default()
                },
            ));

            for (role, names) in CREDITS {
                parent
                    .spawn(Node {
                        // Top-aligned so a role with several names keeps its
                        // label beside the first of them.
                        align_items: AlignItems::Start,
                        column_gap: px(32),
                        ..default()
                    })
                    .with_children(|row| {
                        // The label sits in its own fixed-width box pushed to the
                        // right, so every role ends on the same column and the
                        // names all start on the next one.
                        spawn_column(row, ROLE_WIDTH, AlignItems::End, |cell| {
                            cell.spawn((
                                Text::new(role),
                                TextFont {
                                    font_size: FontSize::Px(ROLE_FONT),
                                    ..default()
                                },
                                TextLayout::new(Justify::Right, LineBreak::NoWrap),
                                TextColor(MUTED_TEXT),
                                Node {
                                    margin: UiRect::top(px(ROLE_NUDGE)),
                                    ..default()
                                },
                            ));
                        });
                        spawn_column(row, NAME_WIDTH, AlignItems::Start, |cell| {
                            for name in names {
                                cell.spawn((
                                    Text::new(*name),
                                    TextFont {
                                        font_size: FontSize::Px(NAME_FONT),
                                        ..default()
                                    },
                                    TextLayout::new(Justify::Left, LineBreak::NoWrap),
                                    TextColor(Color::WHITE),
                                ));
                            }
                        });
                    });
            }

            for (index, line) in COLOPHON.iter().enumerate() {
                parent.spawn((
                    Text::new(*line),
                    TextFont {
                        font_size: FontSize::Px(20.0),
                        ..default()
                    },
                    TextColor(MUTED_TEXT),
                    Node {
                        // Set the colophon apart from the credit rows above it.
                        margin: UiRect::top(px(if index == 0 { 24.0 } else { 0.0 })),
                        ..default()
                    },
                ));
            }

            spawn_back_button(parent);
        });
}

/// One fixed-width cell of the credits table. Contents stack vertically and are
/// pushed to `align` so the two columns meet in the middle.
fn spawn_column(
    row: &mut ChildSpawnerCommands,
    width: f32,
    align: AlignItems,
    contents: impl FnOnce(&mut ChildSpawnerCommands),
) {
    row.spawn(Node {
        width: px(width),
        flex_direction: FlexDirection::Column,
        align_items: align,
        row_gap: px(NAME_GAP),
        ..default()
    })
    .with_children(contents);
}

pub fn despawn_credits(mut commands: Commands, screen: Query<Entity, With<CreditsScreen>>) {
    for entity in &screen {
        commands.entity(entity).despawn();
    }
}
