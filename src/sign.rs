//! The code board hung inside each room's doorway.
//!
//! The board is the other half of the manual's room index: the readout under
//! the clock names a code, the index says which deck and side that code is on,
//! and this is what confirms it once the player is standing there. Without it
//! the index would be a list a player has to take on trust, and a room they had
//! walked into would look like every other room.
//!
//! Rendered in the world rather than as part of the HUD — it is a sign bolted
//! to a wall, and it goes past out of sight with the rest of the room.

use bevy::prelude::*;

use crate::level::{Level, RoomCodes};

/// Wide enough for four characters and the `#`, at a size that reads from the
/// far side of the room the doorway opens onto.
const BOARD_SIZE: Vec2 = Vec2::new(116.0, 44.0);
const CODE_FONT: f32 = 26.0;

/// Darker than the hull it is bolted to, so the board reads as a fitting rather
/// than as a patch of wall.
const BOARD_COLOR: Color = Color::srgb(0.14, 0.15, 0.19);
const BORDER_COLOR: Color = Color::srgb(0.32, 0.35, 0.42);
const CODE_COLOR: Color = Color::srgb(0.85, 0.87, 0.92);

const BORDER: f32 = 3.0;

/// Behind the player and the crates, in front of the hull tiles — the same
/// place the isolation panel's backplate sits, for the same reason.
const BORDER_Z: f32 = -1.62;
const BOARD_Z: f32 = -1.61;
const CODE_Z: f32 = -1.6;

pub fn spawn_room_signs(
    commands: &mut Commands,
    codes: &RoomCodes,
    level: Level,
    marker: impl Bundle + Clone,
) {
    for room in level.rooms() {
        let at = room.sign();

        commands.spawn((
            marker.clone(),
            Sprite {
                color: BORDER_COLOR,
                custom_size: Some(BOARD_SIZE),
                ..default()
            },
            Transform::from_xyz(at.x, at.y, BORDER_Z),
        ));
        commands.spawn((
            marker.clone(),
            Sprite {
                color: BOARD_COLOR,
                custom_size: Some(BOARD_SIZE - Vec2::splat(BORDER * 2.0)),
                ..default()
            },
            Transform::from_xyz(at.x, at.y, BOARD_Z),
        ));
        commands.spawn((
            marker.clone(),
            Text2d::new(format!("#{}", codes.of(*room))),
            TextFont {
                font_size: FontSize::Px(CODE_FONT),
                ..default()
            },
            TextColor(CODE_COLOR),
            Transform::from_xyz(at.x, at.y, CODE_Z),
        ));
    }
}
