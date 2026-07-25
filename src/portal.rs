use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use rand::Rng;

use crate::level::Level;
use crate::minigames::queue_minigame;
use crate::player::Player;
use crate::state::PlayingState;

const PORTAL_RADIUS: f32 = 42.0;
const PORTAL_TEXTURE_SIZE: u32 = 96;
const PORTAL_PULSE_SPEED: f32 = 4.8;
const PORTAL_PULSE_MIN_SCALE: f32 = 0.9;
const PORTAL_PULSE_MAX_SCALE: f32 = 1.14;

#[derive(Component, Clone, Copy)]
pub struct Portal {
    pub minigame: crate::minigames::MinigameId,
    pub used: bool,
}

#[derive(Component)]
pub struct PortalPulse;

pub fn spawn_portals(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    level: Level,
    marker: impl Bundle + Clone,
) {
    let positions = level.portals();
    let minigames = level.portal_minigames();

    if positions.is_empty() || minigames.is_empty() {
        return;
    }

    let texture = broken_portal_texture(images);
    let start_index = random_index(minigames.len());

    for (index, position) in positions.iter().enumerate() {
        let minigame = minigames[(start_index + index) % minigames.len()];

        commands.spawn((
            marker.clone(),
            Portal { minigame, used: false },
            PortalPulse,
            Sprite {
                image: texture.clone(),
                custom_size: Some(Vec2::splat(PORTAL_RADIUS * 2.0)),
                ..default()
            },
            Transform::from_xyz(position.x, position.y, 2.0),
        ));
    }
}

pub fn enter_portal(
    mut commands: Commands,
    players: Query<&Transform, With<Player>>,
    mut portals: Query<(&Transform, &mut Portal)>,
    mut next_playing: ResMut<NextState<PlayingState>>,
) {
    let Some(player) = players.iter().next() else {
        return;
    };

    let player_pos = player.translation.truncate();
    let nearest = portals
        .iter_mut()
        .filter(|(portal_transform, portal)| {
            !portal.used && portal_transform.translation.truncate().distance(player_pos) <= PORTAL_RADIUS
        })
        .min_by(|(one_transform, _), (other_transform, _)| {
            one_transform
                .translation
                .truncate()
                .distance_squared(player_pos)
                .total_cmp(&other_transform.translation.truncate().distance_squared(player_pos))
        });

    let Some((_, mut portal)) = nearest else {
        return;
    };

    portal.used = true;
    queue_minigame(
        &mut commands,
        crate::minigames::MinigameConfig { id: portal.minigame },
    );
    next_playing.set(PlayingState::Minigame);
}

fn random_index(len: usize) -> usize {
    if len == 0 {
        return 0;
    }

    // Use the RNG crate instead of wall-clock modulo so initial picks are unbiased.
    rand::thread_rng().gen_range(0..len)
}

pub fn pulse_portal(
    time: Res<Time>,
    mut portals: Query<(&mut Transform, &mut Sprite), With<PortalPulse>>,
) {
    let phase = time.elapsed_secs() * PORTAL_PULSE_SPEED;
    let pulse = phase.sin() * 0.5 + 0.5;
    let scale = PORTAL_PULSE_MIN_SCALE + pulse * (PORTAL_PULSE_MAX_SCALE - PORTAL_PULSE_MIN_SCALE);

    for (index, (mut transform, mut sprite)) in portals.iter_mut().enumerate() {
        let time_bias = time.elapsed_secs() + index as f32 * 0.73;
        let x_jitter = 0.06 * (time_bias * 17.0).sin() + 0.02 * (time_bias * 33.0).sin();
        let y_jitter = 0.05 * (time_bias * 19.0 + 0.7).sin() + 0.015 * (time_bias * 41.0).sin();

        transform.scale = Vec3::new(scale * (1.0 + x_jitter), scale * (1.0 + y_jitter), 1.0);
        transform.rotation = Quat::from_rotation_z(0.03 * (time_bias * 13.0).sin());

        let flicker = (time_bias * 11.0).sin() * 0.5 + 0.5;
        let glitch_drop = ((time_bias * 7.0).sin() * 0.5 + 0.5).powf(10.0);
        let intensity = (0.72 + 0.28 * flicker) * (1.0 - 0.4 * glitch_drop);

        sprite.color = Color::srgba(
            0.85 + 0.15 * intensity,
            0.2 + 0.25 * intensity,
            0.15 + 0.2 * intensity,
            0.75 + 0.25 * intensity,
        );
    }
}

fn broken_portal_texture(images: &mut Assets<Image>) -> Handle<Image> {
    let size = PORTAL_TEXTURE_SIZE as usize;
    let mut data = vec![0_u8; size * size * 4];

    let center = PORTAL_TEXTURE_SIZE as f32 * 0.5;
    let inv_radius = 1.0 / (center - 1.0);

    for y in 0..PORTAL_TEXTURE_SIZE {
        for x in 0..PORTAL_TEXTURE_SIZE {
            let dx = x as f32 + 0.5 - center;
            let dy = y as f32 + 0.5 - center;
            let nx = dx * inv_radius;
            let ny = dy * inv_radius;
            let radius = (nx * nx + ny * ny).sqrt();

            if radius > 1.0 {
                continue;
            }

            let angle = ny.atan2(nx);
            let ring_center = 0.58
                + 0.06 * (angle * 5.0).sin()
                + 0.03 * (angle * 11.0 + radius * 14.0).sin();
            let ring_thickness = 0.12 + 0.03 * (angle * 9.0).cos();
            let ring_distance = (radius - ring_center).abs();

            let mut energy = 0.0_f32;
            if ring_distance < ring_thickness {
                energy = 1.0 - ring_distance / ring_thickness;
            }

            let pixel_hash = hash2(x, y);
            let spark_band = radius > 0.35 && radius < 0.9 && (y % 3 == 0 || x % 5 == 0);
            if spark_band && pixel_hash > 0.92 {
                energy = energy.max(0.55 + (pixel_hash - 0.92) * 5.0);
            }

            // Creates horizontal interference bars so it reads as broken electronics.
            if y % 6 == 0 {
                energy *= 0.65;
            }

            // Randomly carve out a few arc segments to avoid a clean ellipse.
            let outage = ((angle * 3.5 + radius * 18.0).sin() * 0.5 + 0.5) * pixel_hash;
            if outage > 0.92 {
                energy *= 0.25;
            }

            if energy <= 0.01 {
                continue;
            }

            let i = ((y as usize * size) + x as usize) * 4;
            let orange = (220.0 * energy).clamp(0.0, 255.0) as u8;
            let red = (255.0 * (0.3 + 0.7 * energy)).clamp(0.0, 255.0) as u8;
            let blue = (70.0 * (0.2 + 0.8 * energy)).clamp(0.0, 255.0) as u8;
            let alpha = (255.0 * energy).clamp(0.0, 255.0) as u8;

            data[i] = red;
            data[i + 1] = orange;
            data[i + 2] = blue;
            data[i + 3] = alpha;
        }
    }

    images.add(Image::new_fill(
        Extent3d {
            width: PORTAL_TEXTURE_SIZE,
            height: PORTAL_TEXTURE_SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &data,
        TextureFormat::Rgba8UnormSrgb,
        bevy::asset::RenderAssetUsages::RENDER_WORLD,
    ))
}

fn hash2(x: u32, y: u32) -> f32 {
    let mut n = x.wrapping_mul(374_761_393) ^ y.wrapping_mul(668_265_263);
    n = (n ^ (n >> 13)).wrapping_mul(1_274_126_177);
    let n = n ^ (n >> 16);
    n as f32 / u32::MAX as f32
}
