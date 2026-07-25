use bevy::prelude::*;

use super::{MinigameInstance, MinigameOutcome};

const SEQUENCE: [KeyCode; 4] = [KeyCode::Space, KeyCode::KeyE, KeyCode::Space, KeyCode::KeyE];

pub struct SequenceChallenge {
    step: usize,
}

impl SequenceChallenge {
    pub fn new() -> Self {
        Self { step: 0 }
    }
}

impl MinigameInstance for SequenceChallenge {
    fn title(&self) -> &'static str {
        "Circuit Sequencer"
    }

    fn instructions(&self) -> &'static str {
        "Tap SPACE, E, SPACE, E in order."
    }

    fn status(&self) -> String {
        format!("Sequence: {}/{}", self.step, SEQUENCE.len())
    }

    fn tick(&mut self, keys: &ButtonInput<KeyCode>) -> Option<MinigameOutcome> {
        let Some(expected) = SEQUENCE.get(self.step) else {
            return Some(MinigameOutcome::Success);
        };

        if keys.just_pressed(*expected) {
            self.step += 1;
            if self.step >= SEQUENCE.len() {
                return Some(MinigameOutcome::Success);
            }
        }

        None
    }
}