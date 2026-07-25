use bevy::prelude::*;

use super::{MinigameInstance, MinigameOutcome};

const GOAL_TAPS: u32 = 14;

pub struct TapChallenge {
    taps: u32,
}

impl TapChallenge {
    pub fn new() -> Self {
        Self { taps: 0 }
    }
}

impl MinigameInstance for TapChallenge {
    fn title(&self) -> &'static str {
        "Core Relay Bypass"
    }

    fn instructions(&self) -> &'static str {
        "Tap SPACE repeatedly before time runs out."
    }

    fn status(&self, remaining_seconds: f32) -> String {
        format!(
            "Pulses: {}/{}   Time: {:.1}s",
            self.taps,
            GOAL_TAPS,
            remaining_seconds.max(0.0)
        )
    }

    fn tick(&mut self, keys: &ButtonInput<KeyCode>) -> Option<MinigameOutcome> {
        if keys.just_pressed(KeyCode::Space) {
            self.taps += 1;
            if self.taps >= GOAL_TAPS {
                return Some(MinigameOutcome::Success);
            }
        }

        None
    }
}
