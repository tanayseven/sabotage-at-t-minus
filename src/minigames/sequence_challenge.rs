use bevy::prelude::*;

use super::{
    MinigameAudioCue, MinigameInstance, MinigameOutcome, MinigameVisualState,
    SequenceWireVisualState,
};

const START_SEPARATION: f32 = 120.0;
const MAX_SEPARATION: f32 = 240.0;
const SPREAD_SPEED: f32 = 58.0;
const PULL_PER_PRESS: f32 = 18.0;
const CLOSE_ENOUGH_SEPARATION: f32 = 1.0;
const JOINT_HOLD_SECONDS: f32 = 1.0;

#[derive(Debug, Clone, Copy)]
enum SequencePhase {
    Pulling,
    JointedArmingZap { remaining: f32 },
    Jointed { remaining: f32 },
}

pub struct SequenceChallenge {
    separation: f32,
    phase: SequencePhase,
}

impl SequenceChallenge {
    pub fn new() -> Self {
        Self {
            separation: START_SEPARATION,
            phase: SequencePhase::Pulling,
        }
    }
}

impl MinigameInstance for SequenceChallenge {
    fn title(&self) -> &'static str {
        "Circuit Sequencer"
    }

    fn instructions(&self) -> &'static str {
        "Mash A and D rapidly to pull the broken wires together."
    }

    fn status(&self) -> String {
        match self.phase {
            SequencePhase::Pulling => "Pull the wires together!".to_string(),
            SequencePhase::JointedArmingZap { .. } => "Wire joint stabilized.".to_string(),
            SequencePhase::Jointed { .. } => "Wire joint stabilized.".to_string(),
        }
    }

    fn visual_state(&self) -> MinigameVisualState {
        MinigameVisualState::SequenceWires(SequenceWireVisualState {
            separation: self.separation,
            jointed: matches!(
                self.phase,
                SequencePhase::JointedArmingZap { .. } | SequencePhase::Jointed { .. }
            ),
        })
    }

    fn take_audio_cues(&mut self) -> Vec<MinigameAudioCue> {
        if let SequencePhase::JointedArmingZap { remaining } = self.phase {
            self.phase = SequencePhase::Jointed { remaining };
            return vec![MinigameAudioCue::SequenceZap];
        }

        Vec::new()
    }

    fn tick(&mut self, keys: &ButtonInput<KeyCode>, delta_seconds: f32) -> Option<MinigameOutcome> {
        match &mut self.phase {
            SequencePhase::Pulling => {
                self.separation = (self.separation + SPREAD_SPEED * delta_seconds).min(MAX_SEPARATION);

                let mut pulls = 0.0;
                if keys.just_pressed(KeyCode::KeyA) {
                    pulls += 1.0;
                }
                if keys.just_pressed(KeyCode::KeyD) {
                    pulls += 1.0;
                }

                self.separation = (self.separation - pulls * PULL_PER_PRESS).max(0.0);

                if self.separation <= CLOSE_ENOUGH_SEPARATION {
                    self.separation = 0.0;
                    self.phase = SequencePhase::JointedArmingZap {
                        remaining: JOINT_HOLD_SECONDS,
                    };
                }
            }
            SequencePhase::JointedArmingZap { remaining } | SequencePhase::Jointed { remaining } => {
                *remaining -= delta_seconds;
                if *remaining <= 0.0 {
                    self.separation = 0.0;
                    return Some(MinigameOutcome::Success);
                }
            }
        }

        None
    }
}