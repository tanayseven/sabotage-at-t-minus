use bevy::prelude::*;
use rand::Rng;

use crate::minigames::queue_minigame;
use crate::player::Player;
use crate::puzzles::RocketPuzzles;
use crate::state::PlayingState;
use crate::tiles::load_pixel_art;

/// How close the player has to come to a breach for it to take them. Also half
/// the width it is drawn at.
pub const PORTAL_RADIUS: f32 = 42.0;
const PORTAL_SPRITE: &str = "portal.png";
const PORTAL_PULSE_SPEED: f32 = 4.8;
const PORTAL_PULSE_MIN_SCALE: f32 = 0.9;
const PORTAL_PULSE_MAX_SCALE: f32 = 1.14;
const SPARK_EMIT_RATE: f32 = 22.0;
const SPARK_MIN_SPEED: f32 = 45.0;
const SPARK_MAX_SPEED: f32 = 160.0;
const SPARK_MIN_LIFETIME: f32 = 0.09;
const SPARK_MAX_LIFETIME: f32 = 0.26;
const SPARK_MIN_SIZE: f32 = 1.5;
const SPARK_MAX_SIZE: f32 = 3.4;
/// Sparks are struck off the breach, so they are drawn out of its own palette:
/// pale where they come off it, cooling into the deep blue as they fly.
const SPARK_HOT: Color = Color::srgba(0.72, 0.95, 1.0, 0.95);
const SPARK_COOL: Color = Color::srgba(0.16, 0.42, 0.85, 1.0);

#[derive(Component, Clone, Copy)]
pub struct Portal {
    pub minigame: crate::minigames::MinigameId,
    pub used: bool,
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct TriggeredPortal(pub Entity);

#[derive(Component)]
pub struct PortalPulse;

#[derive(Component)]
pub struct PortalSpark {
    velocity: Vec2,
    lifetime: f32,
    start_lifetime: f32,
    start_size: f32,
}

pub fn spawn_portals(
    commands: &mut Commands,
    assets: &AssetServer,
    puzzles: RocketPuzzles,
    marker: impl Bundle + Clone,
) {
    let texture = load_pixel_art(assets, PORTAL_SPRITE);

    for (position, minigame) in puzzles.portal_placements() {
        commands.spawn((
            marker.clone(),
            Portal {
                minigame,
                used: false,
            },
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
    mut portals: Query<(Entity, &Transform, &mut Portal)>,
    mut next_playing: ResMut<NextState<PlayingState>>,
) {
    let Some(player) = players.iter().next() else {
        return;
    };

    let player_pos = player.translation.truncate();
    let nearest = portals
        .iter_mut()
        .filter(|(_, portal_transform, portal)| {
            !portal.used
                && portal_transform.translation.truncate().distance(player_pos) <= PORTAL_RADIUS
        })
        .min_by(|(_, one_transform, _), (_, other_transform, _)| {
            one_transform
                .translation
                .truncate()
                .distance_squared(player_pos)
                .total_cmp(
                    &other_transform
                        .translation
                        .truncate()
                        .distance_squared(player_pos),
                )
        });

    let Some((entity, _, mut portal)) = nearest else {
        return;
    };

    portal.used = true;
    commands.insert_resource(TriggeredPortal(entity));
    queue_minigame(
        &mut commands,
        crate::minigames::MinigameConfig {
            id: portal.minigame,
        },
    );
    next_playing.set(PlayingState::Minigame);
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
        let intensity = (0.78 + 0.22 * flicker) * (1.0 - 0.4 * glitch_drop);

        // White, so the breach keeps the artwork's own colours: only how far in
        // it is faded says whether it is arcing or dropping out.
        sprite.color = Color::srgba(1.0, 1.0, 1.0, intensity);
    }
}

pub fn emit_portal_sparks(
    mut commands: Commands,
    time: Res<Time>,
    portals: Query<&Transform, (With<PortalPulse>, With<Portal>)>,
) {
    let mut rng = rand::thread_rng();
    let dt = time.delta_secs();
    let chance = (SPARK_EMIT_RATE * dt).clamp(0.0, 1.0);

    for transform in &portals {
        if rng.gen_range(0.0..1.0) > chance {
            continue;
        }

        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        let radial = rng.gen_range(PORTAL_RADIUS * 0.55..PORTAL_RADIUS * 0.95);
        let offset = Vec2::from_angle(angle) * radial;

        let lifetime = rng.gen_range(SPARK_MIN_LIFETIME..SPARK_MAX_LIFETIME);
        let speed = rng.gen_range(SPARK_MIN_SPEED..SPARK_MAX_SPEED);
        let drift = Vec2::from_angle(angle + rng.gen_range(-0.45..0.45)) * speed;
        let size = rng.gen_range(SPARK_MIN_SIZE..SPARK_MAX_SIZE);

        commands.spawn((
            PortalSpark {
                velocity: drift,
                lifetime,
                start_lifetime: lifetime,
                start_size: size,
            },
            Sprite {
                color: SPARK_HOT,
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
            Transform::from_xyz(
                transform.translation.x + offset.x,
                transform.translation.y + offset.y,
                transform.translation.z + 0.5,
            ),
        ));
    }
}

pub fn update_portal_sparks(
    mut commands: Commands,
    time: Res<Time>,
    mut sparks: Query<(Entity, &mut PortalSpark, &mut Transform, &mut Sprite)>,
) {
    let dt = time.delta_secs();

    for (entity, mut spark, mut transform, mut sprite) in &mut sparks {
        spark.lifetime -= dt;
        if spark.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        transform.translation.x += spark.velocity.x * dt;
        transform.translation.y += spark.velocity.y * dt;
        spark.velocity *= 0.92;

        let life = spark.lifetime / spark.start_lifetime;
        let alpha = (life * life).clamp(0.0, 1.0);
        let heat = (1.0 - life).clamp(0.0, 1.0);
        let size = spark.start_size * (0.8 + life * 0.5);

        sprite.custom_size = Some(Vec2::splat(size));
        sprite.color = SPARK_HOT.mix(&SPARK_COOL, heat).with_alpha(alpha);
    }
}

#[cfg(test)]
mod tests {
    use super::PORTAL_SPRITE;
    use crate::tiles::assert_art_exists;

    #[test]
    fn the_breach_art_is_where_it_is_asked_for() {
        assert_art_exists(PORTAL_SPRITE);
    }
}
