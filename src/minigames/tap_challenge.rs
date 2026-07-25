use bevy::prelude::*;

use super::{MinigameInstance, MinigameOutcome};

const GOAL_TAPS: u32 = 14;
const TIME_LIMIT_SECONDS: f32 = 8.0;

pub struct TapChallenge {
    taps: u32,
    remaining: f32,
}

impl TapChallenge {
    pub fn new() -> Self {
        Self {
            taps: 0,
            remaining: TIME_LIMIT_SECONDS,
        }
    }
}

impl MinigameInstance for TapChallenge {
    fn title(&self) -> &'static str {
        "Core Relay Bypass"
    }

    fn instructions(&self) -> &'static str {
        "Tap SPACE repeatedly before time runs out."
    }

    fn status(&self) -> String {
        format!(
            "Pulses: {}/{}   Time: {:.1}s",
            self.taps,
            GOAL_TAPS,
            self.remaining.max(0.0)
        )
    }

    fn tick(&mut self, keys: &ButtonInput<KeyCode>, delta_seconds: f32) -> Option<MinigameOutcome> {
        if keys.just_pressed(KeyCode::Space) {
            self.taps += 1;
            if self.taps >= GOAL_TAPS {
                return Some(MinigameOutcome::Success);
            }
        }

        self.remaining -= delta_seconds;
        if self.remaining <= 0.0 {
            Some(MinigameOutcome::Failure)
        } else {
            None
        }
    }
}
