use std::f32::consts::TAU;

use bevy::prelude::*;

use crate::config::{MISSION_SECONDS, URGENT_SECONDS, WARNING_SECONDS};
use crate::state::PlayingState;
use crate::ui::{ACCENT, CLOCK_GREEN, CLOCK_ORANGE};

/// How fast the readout beats, in pulses per second, once time is critical.
const HEARTBEAT_HZ: f32 = 2.0;
/// How far the heartbeat pulse scales the readout up from its base size.
const HEARTBEAT_SCALE: f32 = 0.18;

/// The clock for the current level. It keeps ticking through portal minigames,
/// but pauses behind confirm-quit and other non-gameplay overlays.
#[derive(Resource)]
pub struct MissionTimer(Timer);

impl Default for MissionTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(MISSION_SECONDS, TimerMode::Once))
    }
}

/// The `T-mm:ss` readout under the HUD's quit button. Carries its own base
/// font size so the heartbeat pulse has an unscaled size to pulse from.
#[derive(Component)]
pub struct CountdownLabel {
    base_font_size: f32,
}

#[allow(dead_code)]
pub fn reset_mission_timer(mut commands: Commands) {
    commands.insert_resource(MissionTimer::default());
}

/// Spawned as part of the HUD, so it lives and dies with the rest of the run.
/// Its row does the centring, so the label itself carries no layout of its own.
pub fn spawn_countdown(parent: &mut ChildSpawnerCommands, font_size: f32) {
    parent.spawn((
        CountdownLabel {
            base_font_size: font_size,
        },
        Text::new(format_remaining(MISSION_SECONDS)),
        TextFont {
            font_size: FontSize::Px(font_size),
            ..default()
        },
        TextColor(CLOCK_GREEN),
    ));
}

pub fn tick_countdown(
    time: Res<Time>,
    mut timer: ResMut<MissionTimer>,
    mut labels: Query<(&CountdownLabel, &mut Text, &mut TextColor, &mut TextFont)>,
    mut next_playing: ResMut<NextState<PlayingState>>,
) {
    let remaining = timer.0.tick(time.delta()).remaining_secs();

    for (label, mut text, mut color, mut font) in &mut labels {
        **text = format_remaining(remaining);
        // The clock goes hot, then critical, for the last stretch, so a player
        // watching the level rather than the corner still notices.
        *color = if remaining <= URGENT_SECONDS {
            ACCENT.into()
        } else if remaining <= WARNING_SECONDS {
            CLOCK_ORANGE.into()
        } else {
            CLOCK_GREEN.into()
        };

        // Below the urgent threshold the readout beats like a heart, faster as
        // the clock gets closer to zero.
        let scale = if remaining <= URGENT_SECONDS && remaining > 0.0 {
            let urgency = 1.0 - remaining / URGENT_SECONDS;
            let hz = HEARTBEAT_HZ + urgency * HEARTBEAT_HZ;
            let pulse = (time.elapsed_secs() * hz * TAU).sin().max(0.0);
            1.0 + pulse * HEARTBEAT_SCALE * (0.5 + urgency * 0.5)
        } else {
            1.0
        };
        font.font_size = FontSize::Px(label.base_font_size * scale);
    }

    if timer.0.is_finished() {
        next_playing.set(PlayingState::GameOver);
    }
}

/// Rounds up, so the readout only shows `T-00:00` once the clock has actually
/// run out rather than for the whole final second.
fn format_remaining(remaining: f32) -> String {
    let total = remaining.max(0.0).ceil() as u32;
    format!("T-{:02}:{:02}", total / 60, total % 60)
}

#[cfg(test)]
mod tests {
    use super::format_remaining;

    #[test]
    fn shows_minutes_and_seconds() {
        assert_eq!(format_remaining(120.0), "T-02:00");
        assert_eq!(format_remaining(65.4), "T-01:06");
        assert_eq!(format_remaining(9.0), "T-00:09");
    }

    #[test]
    fn only_reads_zero_once_the_clock_is_out() {
        assert_eq!(format_remaining(0.2), "T-00:01");
        assert_eq!(format_remaining(0.0), "T-00:00");
        assert_eq!(format_remaining(-1.0), "T-00:00");
    }
}
