use std::f32::consts::{FRAC_PI_2, PI};

use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::prelude::*;

use crate::setup::GameEntity;

const CENTRE_TILE: &str = "walls/vertical-center.png";
const CAP_TILE: &str = "walls/vertical-wall-bottom.png";

#[derive(Clone, Copy)]
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

pub fn load_tiles(assets: &AssetServer) -> TileSet {
    let pixel_art = |settings: &mut ImageLoaderSettings| settings.sampler = ImageSampler::nearest();

    TileSet {
        centre: assets
            .load_builder()
            .with_settings(pixel_art)
            .load(CENTRE_TILE),
        cap: assets
            .load_builder()
            .with_settings(pixel_art)
            .load(CAP_TILE),
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
    pub fn spawn(&self, commands: &mut Commands, tiles: &TileSet) {
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
                GameEntity,
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
