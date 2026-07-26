//! The launch pad: the screen between the menu and the level proper.
//!
//! The rocket stands on the pad and a gantry bridge runs out to its hatch. The
//! player climbs the stairway, walks the bridge, and stepping into the hatch is
//! what starts the run — so the level does not begin until the player boards.

use bevy::prelude::*;

use crate::config::{
    DESIGN_HEIGHT, PLATFORM_HEIGHT, PLAYER_HEIGHT, VIEW_HEIGHT, VIEW_WIDTH, WALL_THICKNESS,
};
use crate::platform::Platform;
use crate::player::{Player, spawn_player};
use crate::state::GameState;
use crate::tiles::{Axis, TileRun, TileSet, load_tiles};
use crate::ui::{ACCENT, MUTED_TEXT};
use crate::wall::spawn_walls;

const ROCKET_SPRITE: &str = "rocket.png";
/// Rasterised from `assets-src/onlyrocket.svg`, trimmed to the artwork, so the
/// sprite's own edges are the rocket's edges and these numbers stay honest.
const ROCKET_PIXELS: Vec2 = Vec2::new(508.0, 1040.0);
/// Down from the top of the sprite to the middle of the hatch, as a fraction of
/// its height. The bridge deck is placed off this, so the two line up whatever
/// size the rocket is drawn at.
const HATCH_FROM_TOP: f32 = 0.3135;

const ROCKET_HEIGHT: f32 = 520.0;
const ROCKET_WIDTH: f32 = ROCKET_HEIGHT * ROCKET_PIXELS.x / ROCKET_PIXELS.y;
const ROCKET_X: f32 = 400.0;

/// The country the pad stands in, behind everything on the screen.
const SKY_SPRITE: &str = "rocket-background.png";
const SKY_PIXELS: Vec2 = Vec2::new(1920.0, 1080.0);
/// Cropped rather than squashed: the shorter side is what gets trimmed, so the
/// horizon stays level whatever shape the window is.
const SKY_SIZE: Vec2 = cover(Vec2::new(VIEW_WIDTH, VIEW_HEIGHT), SKY_PIXELS);
/// The sky is the only thing behind the pad, so a gap at either edge would show
/// the void through it.
const _: () = assert!(SKY_SIZE.x >= VIEW_WIDTH && SKY_SIZE.y >= VIEW_HEIGHT);
const SKY_Z: f32 = -9.0;
/// Dusk rather than noon. The pad is the way into a rocket lit by dim plating,
/// and a full-brightness sky both fights that and washes out the hint text the
/// pad prints across the top of it.
const SKY_TINT: Color = Color::srgb(0.5, 0.54, 0.62);

/// The size to draw `art` at to cover `view` whole without distorting it.
const fn cover(view: Vec2, art: Vec2) -> Vec2 {
    let by_width = view.x / art.x;
    let by_height = view.y / art.y;
    let scale = if by_width > by_height {
        by_width
    } else {
        by_height
    };

    Vec2::new(art.x * scale, art.y * scale)
}

/// Behind the bridge and the player, so the deck reads as running *into* the
/// rocket rather than stopping short of it.
const ROCKET_Z: f32 = -2.0;
const PYLON_Z: f32 = -4.0;

const GROUND_TOP: f32 = -DESIGN_HEIGHT / 2.0;
const ROCKET_BOTTOM: f32 = GROUND_TOP;
const ROCKET_TOP: f32 = ROCKET_BOTTOM + ROCKET_HEIGHT;
const HATCH_Y: f32 = ROCKET_TOP - HATCH_FROM_TOP * ROCKET_HEIGHT;

/// Standing on the deck should put the hatch at the player's middle.
const DECK_TOP: f32 = HATCH_Y - PLAYER_HEIGHT / 2.0;
const DECK_CENTRE_Y: f32 = DECK_TOP - PLATFORM_HEIGHT / 2.0;

const BRIDGE_LEFT: f32 = -20.0;
/// Runs past the rocket's near edge so the deck visibly meets the hull.
const BRIDGE_RIGHT: f32 = ROCKET_X - ROCKET_WIDTH / 4.0;

/// Three even risers from the pad floor up to the deck. The jump arc clears
/// `JUMP_SPEED^2 / (2 * GRAVITY)` = 225px, so each one is comfortably reachable.
const STEP_COUNT: f32 = 3.0;
const STEP_RISE: f32 = (DECK_TOP - GROUND_TOP) / STEP_COUNT;

const PLAYER_START: Vec2 = Vec2::new(-520.0, GROUND_TOP + 120.0);

const PYLON_THICKNESS: f32 = 24.0;
const PYLON_XS: [f32; 2] = [60.0, 240.0];

/// Reaching this far right counts as stepping through the hatch. It is the far
/// end of the deck, which is already inside the rocket's silhouette.
const BOARDING_X: f32 = BRIDGE_RIGHT;
/// How far off the hatch's centre line still counts as going through it.
/// Standing on the deck puts the player dead level with it, so this is slack
/// for a player who boards mid-stride rather than a second way in.
const HATCH_REACH: f32 = PLAYER_HEIGHT;

/// Marks everything the launch pad builds, so leaving the screen clears it.
#[derive(Component, Clone)]
pub struct LaunchpadEntity;

pub fn spawn_launchpad(mut commands: Commands, assets: Res<AssetServer>) {
    let tiles = load_tiles(&assets);

    spawn_sky(&mut commands, &assets);
    spawn_walls(&mut commands, &tiles, LaunchpadEntity);
    spawn_rocket(&mut commands, &assets);
    spawn_gantry(&mut commands, &tiles);
    spawn_player(&mut commands, &assets, PLAYER_START, LaunchpadEntity);
    spawn_launchpad_hud(&mut commands);
}

fn spawn_sky(commands: &mut Commands, assets: &AssetServer) {
    commands.spawn((
        LaunchpadEntity,
        Sprite {
            image: assets.load(SKY_SPRITE),
            color: SKY_TINT,
            custom_size: Some(SKY_SIZE),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, SKY_Z),
    ));
}

fn spawn_rocket(commands: &mut Commands, assets: &AssetServer) {
    commands.spawn((
        LaunchpadEntity,
        Sprite {
            image: assets.load(ROCKET_SPRITE),
            custom_size: Some(Vec2::new(ROCKET_WIDTH, ROCKET_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(ROCKET_X, ROCKET_BOTTOM + ROCKET_HEIGHT / 2.0, ROCKET_Z),
    ));
}

/// The stairway, the bridge deck the player walks to the hatch on, and the
/// pylons holding the deck up.
fn spawn_gantry(commands: &mut Commands, tiles: &TileSet) {
    let steps = [
        Platform {
            centre: Vec2::new(-300.0, GROUND_TOP + STEP_RISE - PLATFORM_HEIGHT / 2.0),
            width: 220.0,
        },
        Platform {
            centre: Vec2::new(-140.0, GROUND_TOP + STEP_RISE * 2.0 - PLATFORM_HEIGHT / 2.0),
            width: 200.0,
        },
    ];
    let bridge = Platform {
        centre: Vec2::new((BRIDGE_LEFT + BRIDGE_RIGHT) / 2.0, DECK_CENTRE_Y),
        width: BRIDGE_RIGHT - BRIDGE_LEFT,
    };

    for platform in steps.into_iter().chain([bridge]) {
        platform.spawn(commands, tiles, LaunchpadEntity);
    }

    // Decoration only: the deck already has its collider, and colliding pylons
    // would just be walls the player bumps into on the way to the rocket.
    let pylon_length = DECK_TOP - PLATFORM_HEIGHT - GROUND_TOP;
    for x in PYLON_XS {
        TileRun {
            centre: Vec2::new(x, GROUND_TOP + pylon_length / 2.0),
            axis: Axis::Vertical,
            length: pylon_length,
            thickness: PYLON_THICKNESS,
            z: PYLON_Z,
        }
        .spawn(commands, tiles, LaunchpadEntity);
    }
}

fn spawn_launchpad_hud(commands: &mut Commands) {
    commands
        .spawn((
            LaunchpadEntity,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(8),
                padding: UiRect::top(px(WALL_THICKNESS + 16.0)),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Launch Pad"),
                TextFont {
                    font_size: FontSize::Px(44.0),
                    ..default()
                },
                TextColor(ACCENT),
            ));
            parent.spawn((
                Text::new("A/D to move, W / space to jump - board the rocket to begin"),
                TextFont {
                    font_size: FontSize::Px(22.0),
                    ..default()
                },
                TextColor(MUTED_TEXT),
            ));
            parent.spawn((
                Text::new("Esc - back to menu"),
                TextFont {
                    font_size: FontSize::Px(18.0),
                    ..default()
                },
                TextColor(MUTED_TEXT),
            ));
        });
}

pub fn board_rocket(
    players: Query<&Transform, With<Player>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let boarded = players.iter().any(|transform| {
        let position = transform.translation.truncate();
        position.x >= BOARDING_X && (position.y - HATCH_Y).abs() <= HATCH_REACH
    });

    if boarded {
        next_state.set(GameState::Playing);
    }
}

pub fn leave_launchpad(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Menu);
    }
}

pub fn despawn_launchpad(mut commands: Commands, entities: Query<Entity, With<LaunchpadEntity>>) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::{ROCKET_SPRITE, SKY_PIXELS, SKY_SIZE, SKY_SPRITE, cover};
    use crate::tiles::assert_art_exists;
    use bevy::prelude::*;

    #[test]
    fn the_pad_s_art_is_where_it_is_asked_for() {
        assert_art_exists(SKY_SPRITE);
        assert_art_exists(ROCKET_SPRITE);
    }

    /// Covering it by stretching would tilt the horizon and squash the clouds.
    #[test]
    fn the_sky_is_cropped_rather_than_distorted() {
        let aspect = |size: Vec2| size.x / size.y;

        assert!((aspect(SKY_SIZE) - aspect(SKY_PIXELS)).abs() < 1e-4);
    }

    /// Whichever axis is the tighter fit is the one met exactly; the other
    /// overhangs and is cropped. Both directions, because a window can be either
    /// wider or taller than the art it is being papered with.
    #[test]
    fn cover_meets_the_tighter_axis_and_overhangs_the_other() {
        let art = Vec2::new(200.0, 100.0);

        assert_eq!(cover(Vec2::new(400.0, 100.0), art), Vec2::new(400.0, 200.0));
        assert_eq!(cover(Vec2::new(200.0, 200.0), art), Vec2::new(400.0, 200.0));
    }
}
