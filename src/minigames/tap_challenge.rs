#![allow(dead_code)]
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
        "Tap SPACE to drive the line. See the repair manual."
    }

    fn status(&self) -> String {
        format!("Pulses: {}/{}", self.taps, GOAL_TAPS)
    }

    fn tick(
        &mut self,
        keys: &ButtonInput<KeyCode>,
        _delta_seconds: f32,
    ) -> Option<MinigameOutcome> {
        if keys.just_pressed(KeyCode::Space) {
            self.taps += 1;
            if self.taps >= GOAL_TAPS {
                return Some(MinigameOutcome::Success);
            }
        }

        None
    }
}
