use std::f32::consts::{FRAC_PI_2, PI};

use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::prelude::*;

const CENTRE_TILE: &str = "walls/vertical-center.png";
const CAP_TILE: &str = "walls/vertical-wall-bottom.png";

#[derive(Debug, Clone, Copy)]
pub enum Axis {
    Vertical,
    Horizontal,
}

impl Axis {
    fn direction(self) -> Vec2 {
        match self {
            Axis::Vertical => Vec2::Y,
            Axis::Horizontal => Vec2::X,
        }
    }

    pub fn half_extents(self, length: f32, thickness: f32) -> Vec2 {
        let half_length = length / 2.0;
        let half_thickness = thickness / 2.0;

        match self {
            Axis::Vertical => Vec2::new(half_thickness, half_length),
            Axis::Horizontal => Vec2::new(half_length, half_thickness),
        }
    }

    fn centre_rotation(self) -> f32 {
        match self {
            Axis::Vertical => 0.0,
            Axis::Horizontal => FRAC_PI_2,
        }
    }

    fn start_cap_rotation(self) -> f32 {
        match self {
            Axis::Vertical => 0.0,
            Axis::Horizontal => -FRAC_PI_2,
        }
    }

    fn end_cap_rotation(self) -> f32 {
        match self {
            Axis::Vertical => PI,
            Axis::Horizontal => FRAC_PI_2,
        }
    }
}

pub struct TileSet {
    centre: Handle<Image>,
    cap: Handle<Image>,
}

/// Loads an image with nearest-neighbour filtering, so the pixel art keeps its
/// hard edges instead of being smeared by the default linear sampler.
pub fn load_pixel_art(assets: &AssetServer, path: &'static str) -> Handle<Image> {
    assets
        .load_builder()
        .with_settings(|settings: &mut ImageLoaderSettings| {
            settings.sampler = ImageSampler::nearest()
        })
        .load(path)
}

/// Holds a module to the art it asks for by name. A renamed file does not break
/// the build — the load simply yields nothing, and the first anyone knows of it
/// is a screen with a hole in it. Shared so each module's own test is one line.
#[cfg(test)]
pub fn assert_art_exists(path: &str) {
    let assets = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");

    assert!(
        assets.join(path).exists(),
        "no asset at assets/{path} — was it renamed?"
    );
}

pub fn load_tiles(assets: &AssetServer) -> TileSet {
    TileSet {
        centre: load_pixel_art(assets, CENTRE_TILE),
        cap: load_pixel_art(assets, CAP_TILE),
    }
}

pub struct TileRun {
    pub centre: Vec2,
    pub axis: Axis,
    pub length: f32,
    pub thickness: f32,
    pub z: f32,
}

impl TileRun {
    /// `marker` is stamped on every tile so the caller's screen can despawn its
    /// own scenery: gameplay tiles carry [`GameEntity`](crate::setup::GameEntity),
    /// the launch pad's carry its own marker.
    pub fn spawn(&self, commands: &mut Commands, tiles: &TileSet, marker: impl Bundle + Clone) {
        let tile_count = (self.length / self.thickness).round().max(2.0);
        let tile_length = self.length / tile_count;
        let tile_count = tile_count as usize;

        let direction = self.axis.direction();
        let first_offset = -(self.length - tile_length) / 2.0;

        for index in 0..tile_count {
            let offset = first_offset + index as f32 * tile_length;
            let position = self.centre + direction * offset;

            let (image, rotation) = if index == 0 {
                (tiles.cap.clone(), self.axis.start_cap_rotation())
            } else if index == tile_count - 1 {
                (tiles.cap.clone(), self.axis.end_cap_rotation())
            } else {
                (tiles.centre.clone(), self.axis.centre_rotation())
            };

            commands.spawn((
                marker.clone(),
                Sprite {
                    image,
                    custom_size: Some(Vec2::new(self.thickness, tile_length)),
                    ..default()
                },
                Transform::from_xyz(position.x, position.y, self.z)
                    .with_rotation(Quat::from_rotation_z(rotation)),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CAP_TILE, CENTRE_TILE, assert_art_exists};

    #[test]
    fn the_wall_tiles_are_where_they_are_asked_for() {
        assert_art_exists(CENTRE_TILE);
        assert_art_exists(CAP_TILE);
    }
}
