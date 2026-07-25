use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::level::Level;
use crate::minigames::{MinigameId, queue_minigame};
use crate::player::Player;
use crate::state::PlayingState;

const PORTAL_RADIUS: f32 = 42.0;
const PORTAL_TEXTURE_SIZE: u32 = 96;
const PORTAL_PULSE_SPEED: f32 = 4.0;
const PORTAL_PULSE_MIN_SCALE: f32 = 0.92;
const PORTAL_PULSE_MAX_SCALE: f32 = 1.10;

#[derive(Component, Clone)]
pub struct Portal;

#[derive(Component)]
pub struct PortalPulse;

#[derive(Resource, Default)]
pub struct PortalState {
    pub used: bool,
}

pub fn reset_portal(mut commands: Commands) {
    commands.insert_resource(PortalState::default());
}

pub fn spawn_portal(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    level: Level,
    marker: impl Bundle,
) {
    let position = level.portal_anchor();
    let texture = red_circle_texture(images);

    commands.spawn((
        marker,
        Portal,
        PortalPulse,
        Sprite {
            image: texture,
            custom_size: Some(Vec2::splat(PORTAL_RADIUS * 2.0)),
            ..default()
        },
        Transform::from_xyz(position.x, position.y, 2.0),
    ));
}

pub fn enter_portal(
    mut commands: Commands,
    mut portal_state: ResMut<PortalState>,
    players: Query<&Transform, With<Player>>,
    portals: Query<&Transform, With<Portal>>,
    mut next_playing: ResMut<NextState<PlayingState>>,
) {
    if portal_state.used {
        return;
    }

    let Some(player) = players.iter().next() else {
        return;
    };

    let player_pos = player.translation.truncate();
    let entered = portals
        .iter()
        .any(|portal| portal.translation.truncate().distance(player_pos) <= PORTAL_RADIUS);

    if entered {
        portal_state.used = true;
        queue_minigame(&mut commands, MinigameId::TapChallenge);
        next_playing.set(PlayingState::Minigame);
    }
}

pub fn pulse_portal(time: Res<Time>, mut portals: Query<&mut Transform, With<PortalPulse>>) {
    let phase = time.elapsed_secs() * PORTAL_PULSE_SPEED;
    let pulse = phase.sin() * 0.5 + 0.5;
    let scale = PORTAL_PULSE_MIN_SCALE + pulse * (PORTAL_PULSE_MAX_SCALE - PORTAL_PULSE_MIN_SCALE);

    for mut transform in &mut portals {
        transform.scale = Vec3::splat(scale);
    }
}

fn red_circle_texture(images: &mut Assets<Image>) -> Handle<Image> {
    let size = PORTAL_TEXTURE_SIZE as usize;
    let mut data = vec![0_u8; size * size * 4];

    let radius = PORTAL_TEXTURE_SIZE as f32 / 2.0 - 1.0;
    let center = PORTAL_TEXTURE_SIZE as f32 / 2.0;

    for y in 0..PORTAL_TEXTURE_SIZE {
        for x in 0..PORTAL_TEXTURE_SIZE {
            let dx = x as f32 + 0.5 - center;
            let dy = y as f32 + 0.5 - center;
            let inside = dx * dx + dy * dy <= radius * radius;
            if inside {
                let i = ((y as usize * size) + x as usize) * 4;
                data[i] = 220;
                data[i + 1] = 36;
                data[i + 2] = 28;
                data[i + 3] = 255;
            }
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
