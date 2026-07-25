use bevy::prelude::*;

use crate::config::{MISSION_SECONDS, URGENT_SECONDS};
use crate::state::PlayingState;
use crate::ui::{ACCENT, MUTED_TEXT};

/// The clock for the current level. It keeps ticking through portal minigames,
/// but pauses behind confirm-quit and other non-gameplay overlays.
#[derive(Resource)]
pub struct MissionTimer(Timer);

impl Default for MissionTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(MISSION_SECONDS, TimerMode::Once))
    }
}

/// The `T-mm:ss` readout under the HUD's quit button.
#[derive(Component)]
pub struct CountdownLabel;

#[allow(dead_code)]
pub fn reset_mission_timer(mut commands: Commands) {
    commands.insert_resource(MissionTimer::default());
}

/// Spawned as part of the HUD, so it lives and dies with the rest of the run.
/// Its row does the centring, so the label itself carries no layout of its own.
pub fn spawn_countdown(parent: &mut ChildSpawnerCommands, font_size: f32) {
    parent.spawn((
        CountdownLabel,
        Text::new(format_remaining(MISSION_SECONDS)),
        TextFont {
            font_size: FontSize::Px(font_size),
            ..default()
        },
        TextColor(MUTED_TEXT),
    ));
}

pub fn tick_countdown(
    time: Res<Time>,
    mut timer: ResMut<MissionTimer>,
    mut labels: Query<(&mut Text, &mut TextColor), With<CountdownLabel>>,
    mut next_playing: ResMut<NextState<PlayingState>>,
) {
    let remaining = timer.0.tick(time.delta()).remaining_secs();

    for (mut text, mut color) in &mut labels {
        **text = format_remaining(remaining);
        // The clock goes hot for the last few seconds, so a player watching the
        // level rather than the corner still notices.
        *color = if remaining <= URGENT_SECONDS {
            ACCENT.into()
        } else {
            MUTED_TEXT.into()
        };
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
