use bevy::prelude::*;
use bevy::text::LineBreak;

use crate::ui::{ACCENT, BACKDROP, GameFont, MUTED_TEXT, spawn_back_button};

/// Names in [`CREDITS`] that point somewhere further — their entry in the
/// table doubles as a link, opened in the system browser when clicked, rather
/// than just being read.
const CREDIT_LINKS: &[(&str, &str)] = &[("Param Siddharth", "https://www.paramsid.com/"), ("Tanay PrabhuDesai", "http://tanay.tech/")];

fn credit_link(name: &str) -> Option<&'static str> {
    CREDIT_LINKS
        .iter()
        .find(|(linked_name, _)| *linked_name == name)
        .map(|(_, url)| *url)
}

/// A credited name that is also a link. Carries the URL rather than reading
/// it back off the label, so a name that happened to repeat elsewhere in the
/// table could never be opened by mistake.
#[derive(Component)]
pub struct CreditLink(&'static str);

/// Wide enough for the longest role and the longest name at their font sizes,
/// so neither column wraps onto a second line.
const ROLE_WIDTH: f32 = 320.0;
const NAME_WIDTH: f32 = 320.0;

const ROLE_FONT: f32 = 24.0;
const NAME_FONT: f32 = 28.0;

/// Vertical spacing between two names sharing a role.
const NAME_GAP: f32 = 4.0;
/// The role label is set in a smaller font than the names, so its line box is
/// shorter. Nudge it down by half the difference to sit level with the first
/// name rather than riding above it.
const ROLE_NUDGE: f32 = (NAME_FONT - ROLE_FONT) * 0.6 / 2.0;

// TODO: replace with the real names before shipping.

/// Each role owns a list of names, printed one per line under a single label.
const CREDITS: [(&str, &[&str]); 3] = [
    (
        "Programming",
        &[
            "Tanay PrabhuDesai",
            "Param Siddharth",
            "Medha Tripathi",
            "Dipesh Joshi",
        ],
    ),
    ("Art", &["Kabir Siddharth"]),
    ("Music", &["Param Siddharth"]),
];

const COLOPHON: [&str; 2] = [
    "Built with Bevy and Rapier",
    "Made for the GMTK Game Jam 2026",
];

#[derive(Component)]
pub struct CreditsScreen;

pub fn spawn_credits(mut commands: Commands, font: Res<GameFont>) {
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
                font.at(56.0),
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
                                let text = (
                                    Text::new(*name),
                                    TextFont {
                                        font_size: FontSize::Px(NAME_FONT),
                                        ..default()
                                    },
                                    TextLayout::new(Justify::Left, LineBreak::NoWrap),
                                );

                                if let Some(url) = credit_link(name) {
                                    // Tinted apart from the plain names beside
                                    // it, so a name worth clicking reads as
                                    // one before it is ever pressed.
                                    cell.spawn((Button, CreditLink(url), text, TextColor(ACCENT)));
                                } else {
                                    cell.spawn((text, TextColor(Color::WHITE)));
                                }
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

/// Opens a clicked credit's link in the system browser. Failures are not this
/// screen's to report — a player without one configured has nowhere for the
/// error to usefully go — so they are only logged.
pub fn open_credit_link_action(links: Query<(&Interaction, &CreditLink), Changed<Interaction>>) {
    for (interaction, link) in &links {
        if *interaction == Interaction::Pressed
            && let Err(error) = webbrowser::open(link.0)
        {
            warn!("could not open {}: {error}", link.0);
        }
    }
}
