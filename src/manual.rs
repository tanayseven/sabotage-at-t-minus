//! The repair manual: the page-by-page book of procedures for undoing the
//! sabotage, opened from the HUD while a run is on.
//!
//! Deliberately *not* modal. Unlike the confirm-quit dialog it is something the
//! player consults mid-job, so opening it neither stands the run down nor stops
//! the clock: the panel is an overlay pinned to the bottom of the screen, the
//! player keeps their controls, and the countdown keeps counting. That is what
//! decides most of what follows — it is not a state, it lives and dies by the
//! presence of one entity, it leaves the HUD and the top of the level clear, and
//! it lets pointer events past everything but its own buttons.
//!
//! What does outlive the panel is the page being read, which is why that sits in
//! [`ManualPage`] rather than in the entities: close the manual to deal with
//! something and it opens again where you left off. The two text nodes are
//! rewritten as the pages turn, so turning a page never rebuilds the layout
//! under the reader.

use bevy::prelude::*;
use bevy::text::{LineBreak, LineHeight};

use crate::panel::Panel;
use crate::ui::{ACCENT, MUTED_TEXT, spawn_button};

/// Not quite opaque: enough of the level shows through to keep the reader in
/// the run, without the text having to fight it.
const PANEL: Color = Color::srgba(0.12, 0.13, 0.17, 0.94);

const NAV_BUTTON_SIZE: Vec2 = Vec2::new(120.0, 40.0);
const NAV_BUTTON_FONT: f32 = 18.0;

/// The shape of a page, in the terms the pages are actually written in — so
/// many lines of so many characters. The box they are set in is derived from
/// these rather than the other way round, and `every_page_fits_its_box` holds
/// the pages to them.
const MAX_PAGE_LINES: usize = 8;
const MAX_LINE_CHARS: usize = 62;

/// A generous upper bound on the default font's average advance per character,
/// as a fraction of the font size. Only used to size the box the text is set
/// in, so erring wide costs a little padding and erring narrow would clip.
const CHAR_ADVANCE: f32 = 0.6;

/// Sized to the longest line the pages are allowed, and held fixed so a short
/// page does not draw a narrower panel than a long one.
const PAGE_WIDTH: f32 = MAX_LINE_CHARS as f32 * BODY_FONT * CHAR_ADVANCE;
/// Sized to the longest page the pages are allowed, so the footer sits on the
/// same line whichever page is open rather than walking up and down as they
/// turn.
const PAGE_HEIGHT: f32 =
    HEADING_FONT * 1.2 + PAGE_ROW_GAP + MAX_PAGE_LINES as f32 * BODY_FONT * BODY_LINE_HEIGHT;
/// Between the page's heading and its body.
const PAGE_ROW_GAP: f32 = 12.0;
/// Between the panel's title, its page and its footer.
const PANEL_ROW_GAP: f32 = 12.0;
const PANEL_PADDING: Vec2 = Vec2::new(32.0, 24.0);
/// How far the panel sits off the bottom edge. It is anchored down there rather
/// than centred so the clock, the HUD and the ground the player is running on
/// all stay in view while the manual is open.
const PANEL_INSET: f32 = 16.0;

const TITLE_FONT: f32 = 26.0;
const HEADING_FONT: f32 = 24.0;
const BODY_FONT: f32 = 19.0;
const BODY_LINE_HEIGHT: f32 = 1.5;

/// One page of the manual: a heading and the lines printed under it.
struct Page {
    heading: &'static str,
    lines: &'static [&'static str],
}

/// Stands in for the line the isolation panel's setting is printed on. The
/// pages are constants and that one line is not — it is decided when the run
/// starts — so it is left as a slot and filled in as the page is drawn.
///
/// A whole line rather than a hole in the middle of one, so the substitution
/// cannot push a line of prose out past the width the page is set to.
const SWITCH_SETTINGS: &str = "\u{0}settings";

/// The manual, in the order the procedures have to be worked. Ordered because
/// the fixes depend on each other — the relay cannot be re-seated while the
/// line behind it is still live — and the pages say so.
const PAGES: [Page; 6] = [
    Page {
        heading: "Read This First",
        lines: &[
            "The rocket leaves at T-00:00 whether or not it is airworthy.",
            "Three systems were tampered with while the pad was clear.",
            "Each has a page. Work them in the order they are printed —",
            "a fix made out of sequence will not hold through ignition.",
            "",
            "Turn pages with the arrow keys. Esc puts the manual away.",
            "The clock does not stop while you have your nose in it.",
        ],
    },
    Page {
        heading: "Isolation Panel",
        lines: &[
            "One room's isolation panel was thrown out of true. Which room",
            "changes with the shift roster — the readout under the clock",
            "names it. Stand at a switch and press E to throw it.",
            "",
            "Left to right, this shift's panel is to be left set:",
            SWITCH_SETTINGS,
            "",
            "The three lamps are wired to the set: they come up together.",
        ],
    },
    Page {
        heading: "1. Coolant Line, Bay 2",
        lines: &[
            "Symptom: frost on the outer skin, pressure reading flat.",
            "",
            "1. Close the bay-2 isolation valve before anything else.",
            "2. Cut away the collapsed section of hose.",
            "3. Fit the spare coupling from the crate on the ledge.",
            "4. Re-open the valve and hold for a two-count.",
            "",
            "The line is right when the frost lifts off the skin.",
        ],
    },
    Page {
        heading: "2. Gimbal Lock, Engine Mount",
        lines: &[
            "Symptom: the nozzle will not swing; steering is dead.",
            "",
            "1. Confirm bay 2 is closed out. The mount shares its loop.",
            "2. Pull the shim that was driven into the gimbal ring.",
            "3. Work the nozzle through its full travel by hand.",
            "4. Torque the ring bolts back down, opposite pairs first.",
            "",
            "Leave the shim on the pad. It is evidence, not spare stock.",
        ],
    },
    Page {
        heading: "3. Ignition Relay, Upper Deck",
        lines: &[
            "Symptom: the arming light stays dark on a good circuit.",
            "",
            "1. Do not touch this until the gimbal swings freely.",
            "2. Unseat the relay. The tab underneath has been filed.",
            "3. Seat the spare, keyed notch toward the hull.",
            "4. Watch for the arming light to hold steady, not flicker.",
            "",
            "A flickering light means the relay is in but not home.",
        ],
    },
    Page {
        heading: "Sign-Off",
        lines: &[
            "Before you clear the pad, walk the three in reverse:",
            "",
            "  Relay seated, arming light steady.",
            "  Nozzle swinging through full travel.",
            "  Coolant pressure up, no frost on the skin.",
            "",
            "Then get clear. The clock does not wait for the paperwork.",
        ],
    },
];

/// The HUD button that opens the manual.
#[derive(Component)]
pub struct ManualButton;

#[derive(Component)]
pub struct ManualScreen;

/// The page heading, rewritten as the pages turn.
#[derive(Component)]
pub struct ManualHeading;

/// The body of the open page, rewritten as the pages turn.
#[derive(Component)]
pub struct ManualBody;

/// The `Page 2 / 6` readout between the two nav buttons.
#[derive(Component)]
pub struct ManualIndicator;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum ManualNav {
    Previous,
    Next,
    Close,
}

/// Which page is open. A resource rather than a component, so it survives the
/// panel being despawned and the manual re-opens where the player left off —
/// mid-run, having to page back to where you were is the last thing you want.
/// It is reset when a run starts, not when the manual closes.
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ManualPage(usize);

impl ManualPage {
    /// Clamped rather than wrapping at both ends: the manual is a sequence of
    /// procedures, and running off the last page back onto the first would read
    /// as having lost your place.
    fn turn(&mut self, by: isize) {
        let last = PAGES.len() - 1;
        self.0 = self.0.saturating_add_signed(by).min(last);
    }

    fn page(self) -> &'static Page {
        // `turn` is the only thing that writes the index, and it clamps.
        &PAGES[self.0]
    }

    fn label(self) -> String {
        format!("Page {} / {}", self.0 + 1, PAGES.len())
    }
}

pub fn reset_manual_page(mut commands: Commands) {
    commands.insert_resource(ManualPage::default());
}

/// `switch_panel` is the isolation panel out in the level, not this one — the
/// manual has a panel of its own, and one of the two had to give way.
fn spawn_manual(commands: &mut Commands, open: ManualPage, switch_panel: &Panel) {
    commands
        .spawn((
            ManualScreen,
            Node {
                position_type: PositionType::Absolute,
                bottom: px(PANEL_INSET),
                width: percent(100),
                justify_content: JustifyContent::Center,
                ..default()
            },
            // The row spans the whole width to centre the panel, which would
            // otherwise lay an invisible node over the level. Only the panel
            // itself takes clicks; everything else falls through to the game.
            Pickable::IGNORE,
            // Above the HUD, so the manual is never half-hidden behind it.
            GlobalZIndex(1),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        padding: UiRect::axes(px(PANEL_PADDING.x), px(PANEL_PADDING.y)),
                        row_gap: px(PANEL_ROW_GAP),
                        border_radius: BorderRadius::all(px(12)),
                        ..default()
                    },
                    BackgroundColor(PANEL),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new("Repair Manual"),
                        TextFont {
                            font_size: FontSize::Px(TITLE_FONT),
                            ..default()
                        },
                        TextColor(ACCENT),
                    ));

                    // The page itself sits in a fixed box, top-aligned, so the
                    // heading holds one line and the body grows downwards into
                    // the slack instead of pushing the panel about.
                    panel
                        .spawn(Node {
                            width: px(PAGE_WIDTH),
                            height: px(PAGE_HEIGHT),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Start,
                            row_gap: px(PAGE_ROW_GAP),
                            ..default()
                        })
                        .with_children(|body| {
                            body.spawn((
                                ManualHeading,
                                Text::new(open.page().heading),
                                TextFont {
                                    font_size: FontSize::Px(HEADING_FONT),
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                            body.spawn((
                                ManualBody,
                                Text::new(body_text(open, switch_panel)),
                                TextFont {
                                    font_size: FontSize::Px(BODY_FONT),
                                    ..default()
                                },
                                // The pages are a wall of short lines, so they
                                // are set looser than the default 1.2 to read
                                // as a printed procedure rather than a block.
                                LineHeight::RelativeToFont(BODY_LINE_HEIGHT),
                                // The pages are written to fit the box, so a
                                // wrap would mean a line has outgrown it rather
                                // than that it needs breaking.
                                TextLayout::new(Justify::Left, LineBreak::NoWrap),
                                TextColor(MUTED_TEXT),
                            ));
                        });

                    panel
                        .spawn(Node {
                            width: px(PAGE_WIDTH),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::SpaceBetween,
                            ..default()
                        })
                        .with_children(|footer| {
                            spawn_button(
                                footer,
                                "< Back",
                                ManualNav::Previous,
                                NAV_BUTTON_SIZE,
                                NAV_BUTTON_FONT,
                            );
                            footer
                                .spawn(Node {
                                    align_items: AlignItems::Center,
                                    column_gap: px(20),
                                    ..default()
                                })
                                .with_children(|middle| {
                                    middle.spawn((
                                        ManualIndicator,
                                        Text::new(open.label()),
                                        TextFont {
                                            font_size: FontSize::Px(20.0),
                                            ..default()
                                        },
                                        TextColor(MUTED_TEXT),
                                    ));
                                    spawn_button(
                                        middle,
                                        "Close",
                                        ManualNav::Close,
                                        NAV_BUTTON_SIZE,
                                        NAV_BUTTON_FONT,
                                    );
                                });
                            spawn_button(
                                footer,
                                "Next >",
                                ManualNav::Next,
                                NAV_BUTTON_SIZE,
                                NAV_BUTTON_FONT,
                            );
                        });
                });
        });
}

/// The open page's body, with the panel's setting printed onto the line the
/// page leaves for it.
fn body_text(page: ManualPage, panel: &Panel) -> String {
    page.page()
        .lines
        .iter()
        .map(|line| {
            if *line == SWITCH_SETTINGS {
                panel.printed_settings()
            } else {
                (*line).to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Opening, closing and paging, in one system because they share a keyboard:
/// `M` toggles, so whether a press opens or closes depends on what the other
/// half would have done with it.
///
/// Runs while the game is running, like any other in-game control, and takes
/// the keyboard as [`ResMut`] so it can eat the keys it acts on. That matters
/// for exactly one of them: `Escape` closes the manual if it is open, and the
/// confirm-quit dialog — which runs after this — must not also see that press
/// and take it as a request to abort the mission.
pub fn manual_controls(
    mut commands: Commands,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut page: ResMut<ManualPage>,
    panel: Res<Panel>,
    open_buttons: Query<&Interaction, (Changed<Interaction>, With<ManualButton>)>,
    nav_buttons: Query<(&Interaction, &ManualNav), Changed<Interaction>>,
    screen: Query<Entity, With<ManualScreen>>,
) {
    let clicked = open_buttons
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed);
    let toggled = clicked || keys.just_pressed(KeyCode::KeyM);

    let Ok(open) = screen.single() else {
        if toggled {
            spawn_manual(&mut commands, *page, &panel);
        }
        return;
    };

    let mut chosen = toggled.then_some(ManualNav::Close);

    for (interaction, nav) in &nav_buttons {
        if *interaction == Interaction::Pressed {
            chosen = Some(*nav);
        }
    }

    if keys.just_pressed(KeyCode::ArrowLeft) {
        chosen = Some(ManualNav::Previous);
    }
    if keys.just_pressed(KeyCode::ArrowRight) {
        chosen = Some(ManualNav::Next);
    }
    if keys.just_pressed(KeyCode::Escape) {
        keys.clear_just_pressed(KeyCode::Escape);
        chosen = Some(ManualNav::Close);
    }

    match chosen {
        // Assigned through `ResMut` so change detection only fires on a page
        // that actually moved — at either end of the book, it does not.
        Some(ManualNav::Previous) => page.turn(-1),
        Some(ManualNav::Next) => page.turn(1),
        Some(ManualNav::Close) => commands.entity(open).despawn(),
        None => {}
    }
}

/// Rewrites the open page. Driven off the resource rather than off the button
/// presses, so the keyboard and the buttons share one path to the screen.
#[allow(clippy::type_complexity)]
pub fn sync_manual_page(
    page: Res<ManualPage>,
    panel: Res<Panel>,
    mut texts: ParamSet<(
        Query<&mut Text, With<ManualHeading>>,
        Query<&mut Text, With<ManualBody>>,
        Query<&mut Text, With<ManualIndicator>>,
    )>,
) {
    // The panel is watched as well as the page: a new run inserts a new one,
    // and the line it prints has to follow it.
    if !page.is_changed() && !panel.is_changed() {
        return;
    }

    for mut text in &mut texts.p0() {
        **text = page.page().heading.to_string();
    }
    for mut text in &mut texts.p1() {
        **text = body_text(*page, &panel);
    }
    for mut text in &mut texts.p2() {
        **text = page.label();
    }
}

/// Puts the manual away whenever the run stops running — the confirm-quit
/// dialog, the game-over screen, or quitting out altogether. Those are modal
/// and the manual is not, so it would otherwise sit on top of them.
pub fn despawn_manual(mut commands: Commands, screens: Query<Entity, With<ManualScreen>>) {
    for entity in &screens {
        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::{
        MAX_LINE_CHARS, MAX_PAGE_LINES, ManualPage, ManualScreen, PAGE_HEIGHT, PAGES,
        SWITCH_SETTINGS, body_text, manual_controls,
    };
    use crate::config::DESIGN_HEIGHT;
    use crate::panel::Panel;

    /// The manual's controls, and nothing else — enough to open and close it
    /// with the keyboard and see what that left behind.
    fn app() -> App {
        let mut app = App::new();
        app.init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ManualPage>()
            .init_resource::<Panel>()
            .add_systems(Update, manual_controls);
        app
    }

    /// One frame with `key` down, then up again. Both halves matter: a key
    /// still held from the last frame does not press a second time, and
    /// `bevy_input` — which is not in this app — is what would otherwise clear
    /// `just_pressed` between frames.
    fn press(app: &mut App, key: KeyCode) {
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(key);
        app.update();

        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.release(key);
        keys.clear();
    }

    fn is_open(app: &mut App) -> bool {
        app.world_mut()
            .query_filtered::<Entity, With<ManualScreen>>()
            .iter(app.world())
            .next()
            .is_some()
    }

    #[test]
    fn m_opens_the_manual_and_puts_it_away_again() {
        let mut app = app();

        press(&mut app, KeyCode::KeyM);
        assert!(is_open(&mut app), "M did not open the manual");

        press(&mut app, KeyCode::KeyM);
        assert!(!is_open(&mut app), "M did not put the manual away");
    }

    #[test]
    fn the_arrow_keys_turn_pages() {
        let mut app = app();

        press(&mut app, KeyCode::KeyM);
        press(&mut app, KeyCode::ArrowRight);
        assert_eq!(app.world().resource::<ManualPage>().0, 1);

        press(&mut app, KeyCode::ArrowLeft);
        assert_eq!(app.world().resource::<ManualPage>().0, 0);
        assert!(is_open(&mut app), "paging closed the manual");
    }

    /// `Escape` is also what opens the confirm-quit dialog, so closing the
    /// manual with it has to swallow the press. Otherwise putting the manual
    /// down in the same breath asks whether to abort the mission.
    #[test]
    fn escape_closes_the_manual_without_reaching_the_quit_dialog() {
        let mut app = app();

        press(&mut app, KeyCode::KeyM);

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        app.update();

        assert!(!is_open(&mut app), "Escape did not close the manual");
        assert!(
            !app.world()
                .resource::<ButtonInput<KeyCode>>()
                .just_pressed(KeyCode::Escape),
            "Escape was left for the quit dialog to find"
        );
    }

    /// With the manual shut, `Escape` is the quit dialog's to read.
    #[test]
    fn escape_is_left_alone_when_the_manual_is_shut() {
        let mut app = app();

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        app.update();

        assert!(
            app.world()
                .resource::<ButtonInput<KeyCode>>()
                .just_pressed(KeyCode::Escape)
        );
    }

    #[test]
    fn a_run_opens_the_manual_on_the_first_page() {
        assert_eq!(ManualPage::default().0, 0);
    }

    #[test]
    fn pages_turn_forwards_and_back() {
        let mut page = ManualPage::default();

        page.turn(1);
        assert_eq!(page.0, 1);
        page.turn(-1);
        assert_eq!(page.0, 0);
    }

    /// Turning past either cover has to leave the reader where they were, not
    /// wrap them round to the other end of the book.
    #[test]
    fn the_book_does_not_wrap_at_either_end() {
        let mut page = ManualPage::default();

        page.turn(-1);
        assert_eq!(page.0, 0, "turned back off the front cover");

        for _ in 0..PAGES.len() + 3 {
            page.turn(1);
        }
        assert_eq!(page.0, PAGES.len() - 1, "turned on past the last page");
    }

    #[test]
    fn the_indicator_counts_from_one() {
        let mut page = ManualPage::default();
        assert_eq!(page.label(), format!("Page 1 / {}", PAGES.len()));

        page.turn(1);
        assert_eq!(page.label(), format!("Page 2 / {}", PAGES.len()));
    }

    /// The page box is a fixed size, so a page that outgrows it would be
    /// clipped rather than resize the panel. These are the bounds the pages
    /// were written to fit.
    #[test]
    fn every_page_fits_its_box() {
        for page in &PAGES {
            assert!(!page.heading.is_empty(), "a page with no heading");
            assert!(!page.lines.is_empty(), "a page with no body");
            assert!(
                page.lines.len() <= MAX_PAGE_LINES,
                "{} runs too long",
                page.heading
            );

            for line in page.lines {
                assert!(
                    line.chars().count() <= MAX_LINE_CHARS,
                    "line too wide for the page: {line:?}"
                );
            }
        }
    }

    /// The one line of the manual that is not written until the run starts. It
    /// still has to fit the box the rest of the page was measured for, whichever
    /// setting the run asks for.
    #[test]
    fn the_setting_printed_for_the_run_fits_the_page() {
        for seed in 0..64u64 {
            let printed = Panel::from_seed(seed).printed_settings();

            assert!(
                printed.chars().count() <= MAX_LINE_CHARS,
                "the printed setting is too wide for the page: {printed:?}"
            );
        }
    }

    /// The point of the page: the manual tells the player what to throw. A page
    /// that still had the slot on it would be one that told them nothing.
    #[test]
    fn the_manual_prints_this_run_s_combination() {
        let panel = Panel::from_seed(11);
        let page = (0..PAGES.len())
            .map(|index| {
                let mut page = ManualPage::default();
                page.turn(index as isize);
                page
            })
            .find(|page| page.page().heading == "Isolation Panel")
            .expect("the manual has no isolation panel page");

        let body = body_text(page, &panel);

        assert!(
            !body.contains(SWITCH_SETTINGS),
            "the page went out with the slot still on it"
        );
        assert!(
            body.contains(&panel.printed_settings()),
            "the page does not print the run's setting"
        );
        for (index, up) in panel.combination.iter().enumerate() {
            let printed = format!("{}: {}", index + 1, if *up { "UP" } else { "DOWN" });

            assert!(body.contains(&printed), "switch {index} is not printed");
        }
    }

    /// Every other page is printed as written, slot or no slot.
    #[test]
    fn the_pages_without_a_setting_are_printed_as_written() {
        let panel = Panel::from_seed(11);
        let mut page = ManualPage::default();

        assert_eq!(body_text(page, &panel), PAGES[0].lines.join("\n"));

        page.turn(PAGES.len() as isize);
        assert_eq!(
            body_text(page, &panel),
            PAGES[PAGES.len() - 1].lines.join("\n")
        );
    }

    /// The manual is read with the run still going on around it, so it has to
    /// leave the top of the screen in view: the mission clock, the HUD, and a
    /// band of the level above them. This is the panel at its tallest, which is
    /// the only height it is ever drawn at.
    #[test]
    fn the_panel_leaves_the_top_of_the_screen_clear() {
        use super::{NAV_BUTTON_SIZE, PANEL_INSET, PANEL_PADDING, PANEL_ROW_GAP, TITLE_FONT};

        /// Room for the HUD's three lines of hint, the countdown beside them,
        /// and enough level above the panel to run by.
        const CLEARANCE: f32 = 240.0;

        let panel = PANEL_PADDING.y * 2.0
            + TITLE_FONT * 1.2
            + PANEL_ROW_GAP
            + PAGE_HEIGHT
            + PANEL_ROW_GAP
            + NAV_BUTTON_SIZE.y
            + PANEL_INSET;

        assert!(
            DESIGN_HEIGHT - panel >= CLEARANCE,
            "the manual leaves only {} of the {DESIGN_HEIGHT}-unit screen clear",
            DESIGN_HEIGHT - panel
        );
    }
}
