use bevy::prelude::*;

use super::{
    MinigameAudioCue, MinigameInstance, MinigameOutcome, MinigameVisualState,
    SequenceWireVisualState,
};

const START_SEPARATION: f32 = 170.0;
const MAX_SEPARATION: f32 = 280.0;
const SPREAD_SPEED: f32 = 82.0;
const PULL_PER_PRESS: f32 = 14.0;
const CLOSE_ENOUGH_SEPARATION: f32 = 0.5;
const JOINT_HOLD_SECONDS: f32 = 1.0;

#[derive(Debug, Clone, Copy)]
enum BrokenWirePhase {
    Pulling,
    JointedArmingZap { remaining: f32 },
    Jointed { remaining: f32 },
}

pub struct BrokenWire {
    separation: f32,
    phase: BrokenWirePhase,
}

impl BrokenWire {
    pub fn new() -> Self {
        Self {
            separation: START_SEPARATION,
            phase: BrokenWirePhase::Pulling,
        }
    }
}

impl MinigameInstance for BrokenWire {
    fn title(&self) -> &'static str {
        "Damaged Signal Relay"
    }

    fn instructions(&self) -> &'static str {
        "Consult the repair manual."
    }

    fn status(&self) -> String {
        match self.phase {
            BrokenWirePhase::Pulling => "Signal unstable.".to_string(),
            BrokenWirePhase::JointedArmingZap { .. } => "Signal stabilizing.".to_string(),
            BrokenWirePhase::Jointed { .. } => "Signal stabilized.".to_string(),
        }
    }

    fn visual_state(&self) -> MinigameVisualState {
        MinigameVisualState::BrokenWires(SequenceWireVisualState {
            separation: self.separation,
            jointed: matches!(
                self.phase,
                BrokenWirePhase::JointedArmingZap { .. } | BrokenWirePhase::Jointed { .. }
            ),
        })
    }

    fn take_audio_cues(&mut self) -> Vec<MinigameAudioCue> {
        if let BrokenWirePhase::JointedArmingZap { remaining } = self.phase {
            self.phase = BrokenWirePhase::Jointed { remaining };
            return vec![MinigameAudioCue::SequenceZap];
        }

        Vec::new()
    }

    fn tick(&mut self, keys: &ButtonInput<KeyCode>, delta_seconds: f32) -> Option<MinigameOutcome> {
        match &mut self.phase {
            BrokenWirePhase::Pulling => {
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
                    self.phase = BrokenWirePhase::JointedArmingZap {
                        remaining: JOINT_HOLD_SECONDS,
                    };
                }
            }
            BrokenWirePhase::JointedArmingZap { remaining } | BrokenWirePhase::Jointed { remaining } => {
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