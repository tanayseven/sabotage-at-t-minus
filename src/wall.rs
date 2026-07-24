use std::f32::consts::{FRAC_PI_2, PI};

use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::config::{DESIGN_HEIGHT, DESIGN_WIDTH, WALL_THICKNESS};
use crate::setup::GameEntity;

const CENTRE_TILE: &str = "walls/vertical-center.png";
const CAP_TILE: &str = "walls/vertical-wall-bottom.png";

const TILE_SIZE: f32 = WALL_THICKNESS;
const WALL_Z: f32 = -5.0;

#[derive(Clone, Copy)]
enum Axis {
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

    fn half_extents(self, length: f32) -> Vec2 {
        let half_length = length / 2.0;
        let half_thickness = WALL_THICKNESS / 2.0;

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

struct WallTiles {
    centre: Handle<Image>,
    cap: Handle<Image>,
}

fn load_wall_tiles(assets: &AssetServer) -> WallTiles {
    let pixel_art = |settings: &mut ImageLoaderSettings| settings.sampler = ImageSampler::nearest();

    WallTiles {
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

struct Wall {
    centre: Vec2,
    axis: Axis,
    length: f32,
}

impl Wall {
    fn spawn(&self, commands: &mut Commands, tiles: &WallTiles) {
        let half_extents = self.axis.half_extents(self.length);

        commands.spawn((
            GameEntity,
            Transform::from_xyz(self.centre.x, self.centre.y, WALL_Z),
            RigidBody::Fixed,
            Collider::cuboid(half_extents.x, half_extents.y),
        ));

        self.spawn_tiles(commands, tiles);
    }

    fn spawn_tiles(&self, commands: &mut Commands, tiles: &WallTiles) {
        let tile_count = (self.length / TILE_SIZE).round().max(2.0) as usize;
        let direction = self.axis.direction();
        let first_offset = -(self.length - TILE_SIZE) / 2.0;

        for index in 0..tile_count {
            let offset = first_offset + index as f32 * TILE_SIZE;
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
                    custom_size: Some(Vec2::splat(TILE_SIZE)),
                    ..default()
                },
                Transform::from_xyz(position.x, position.y, WALL_Z)
                    .with_rotation(Quat::from_rotation_z(rotation)),
            ));
        }
    }
}

pub fn spawn_walls(commands: &mut Commands, assets: &AssetServer) {
    let tiles = load_wall_tiles(assets);

    let half_width = DESIGN_WIDTH / 2.0;
    let half_height = DESIGN_HEIGHT / 2.0;
    let half_thickness = WALL_THICKNESS / 2.0;

    let horizontal_length = DESIGN_WIDTH + WALL_THICKNESS * 2.0;
    let vertical_length = DESIGN_HEIGHT + WALL_THICKNESS * 2.0;

    let top = Wall {
        centre: Vec2::new(0.0, half_height + half_thickness),
        axis: Axis::Horizontal,
        length: horizontal_length,
    };
    let ground = Wall {
        centre: Vec2::new(0.0, -half_height - half_thickness),
        axis: Axis::Horizontal,
        length: horizontal_length,
    };
    let left = Wall {
        centre: Vec2::new(-half_width - half_thickness, 0.0),
        axis: Axis::Vertical,
        length: vertical_length,
    };
    let right = Wall {
        centre: Vec2::new(half_width + half_thickness, 0.0),
        axis: Axis::Vertical,
        length: vertical_length,
    };

    for wall in [top, ground, left, right] {
        wall.spawn(commands, &tiles);
    }
}
