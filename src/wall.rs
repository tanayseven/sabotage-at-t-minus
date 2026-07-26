use bevy::prelude::*;
use bevy::sprite::SpriteImageMode;
use bevy_rapier2d::prelude::*;

use crate::config::{DESIGN_HEIGHT, DESIGN_WIDTH, WALL_THICKNESS};
use crate::tiles::{Axis, TileRun, TileSet, load_pixel_art};

const WALL_Z: f32 = -5.0;

/// The plating the rooms are papered with. One seamless square, laid end to end
/// across the whole interior.
const LINING_SPRITE: &str = "walls/tiles.png";
const LINING_PIXELS: f32 = 256.0;
/// How big one plate is drawn. Well under the 256px the art is drawn at, so the
/// wall reads as a run of panels behind the player rather than one flat slab.
const LINING_PLATE: f32 = 64.0;
/// Drawn at its own size or larger the plating would tile once across a room,
/// if at all, and the texture it is there to carry would be lost.
const _: () = assert!(LINING_PLATE < LINING_PIXELS);
/// Behind the walls, the platforms and the props, but in front of the flat
/// backdrop the camera puts up.
const LINING_Z: f32 = -9.0;
/// Knocked well back, to about what the flat backdrop behind it reads at. The
/// plating is there to give the rooms texture, not to light them: drawn at its
/// own brightness it comes forward over the level and leaves the HUD — which is
/// pale text with nothing behind it — unreadable across the top of the screen.
const LINING_TINT: Color = Color::srgb(0.26, 0.28, 0.34);

/// A solid run of wall. The launch pad and the level proper both build theirs
/// out of these: the pad's four make a box around the viewport, the rocket's
/// make its hull and the bulkheads between its rooms.
#[derive(Debug, Clone, Copy)]
pub struct Wall {
    pub centre: Vec2,
    pub axis: Axis,
    pub length: f32,
}

impl Wall {
    pub const fn new(centre: Vec2, axis: Axis, length: f32) -> Self {
        Self {
            centre,
            axis,
            length,
        }
    }

    /// A vertical wall spanning `bottom` to `top`. Built from its two ends
    /// rather than a centre and a length because that is how a hull and a
    /// bulkhead are actually described — this deck up to that one.
    pub const fn between(x: f32, bottom: f32, top: f32) -> Self {
        Self::new(
            Vec2::new(x, (bottom + top) / 2.0),
            Axis::Vertical,
            top - bottom,
        )
    }

    fn spawn(&self, commands: &mut Commands, tiles: &TileSet, marker: impl Bundle + Clone) {
        let half_extents = self.axis.half_extents(self.length, WALL_THICKNESS);

        commands.spawn((
            marker.clone(),
            Transform::from_xyz(self.centre.x, self.centre.y, WALL_Z),
            RigidBody::Fixed,
            Collider::cuboid(half_extents.x, half_extents.y),
        ));

        TileRun {
            centre: self.centre,
            axis: self.axis,
            length: self.length,
            thickness: WALL_THICKNESS,
            z: WALL_Z,
        }
        .spawn(commands, tiles, marker);
    }
}

pub fn spawn_wall_run(
    commands: &mut Commands,
    tiles: &TileSet,
    walls: &[Wall],
    marker: impl Bundle + Clone,
) {
    for wall in walls {
        wall.spawn(commands, tiles, marker.clone());
    }
}

/// Papers `over` with the rocket's wall plating: one tiled sprite rather than a
/// grid of them, so the whole interior costs a single entity however far the
/// level runs.
pub fn spawn_hull_lining(
    commands: &mut Commands,
    assets: &AssetServer,
    over: Rect,
    marker: impl Bundle,
) {
    commands.spawn((
        marker,
        Sprite {
            image: load_pixel_art(assets, LINING_SPRITE),
            color: LINING_TINT,
            custom_size: Some(over.size()),
            image_mode: SpriteImageMode::Tiled {
                tile_x: true,
                tile_y: true,
                // Read as a fraction of the art's own size, so this is what
                // scales a 256px plate down to `LINING_PLATE`.
                stretch_value: LINING_PLATE / LINING_PIXELS,
            },
            ..default()
        },
        Transform::from_xyz(over.center().x, over.center().y, LINING_Z),
    ));
}

/// The box that frames the play area. The launch pad is fought inside it, so
/// the caller says which screen owns the walls.
pub fn spawn_walls(commands: &mut Commands, tiles: &TileSet, marker: impl Bundle + Clone) {
    let half_width = DESIGN_WIDTH / 2.0;
    let half_height = DESIGN_HEIGHT / 2.0;
    let half_thickness = WALL_THICKNESS / 2.0;

    let horizontal_length = DESIGN_WIDTH + WALL_THICKNESS * 2.0;
    let vertical_length = DESIGN_HEIGHT + WALL_THICKNESS * 2.0;

    let box_walls = [
        Wall::new(
            Vec2::new(0.0, half_height + half_thickness),
            Axis::Horizontal,
            horizontal_length,
        ),
        Wall::new(
            Vec2::new(0.0, -half_height - half_thickness),
            Axis::Horizontal,
            horizontal_length,
        ),
        Wall::new(
            Vec2::new(-half_width - half_thickness, 0.0),
            Axis::Vertical,
            vertical_length,
        ),
        Wall::new(
            Vec2::new(half_width + half_thickness, 0.0),
            Axis::Vertical,
            vertical_length,
        ),
    ];

    spawn_wall_run(commands, tiles, &box_walls, marker);
}

#[cfg(test)]
mod tests {
    use super::LINING_SPRITE;
    use crate::tiles::assert_art_exists;

    #[test]
    fn the_hull_plating_is_where_it_is_asked_for() {
        assert_art_exists(LINING_SPRITE);
    }
}
