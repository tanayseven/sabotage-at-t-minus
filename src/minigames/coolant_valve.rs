use bevy::prelude::*;

use super::{
    CoolantGaugeVisualState, MinigameAudioCue, MinigameInstance, MinigameOutcome,
    MinigameVisualState,
};
use crate::minigame_keys::MinigameKeys;

const PRESSURE_MAX: f32 = 100.0;

/// Painted onto the gauge art at 0.52 and 0.72 of the track. Moving these
/// without redrawing the gauge puts the band somewhere the player cannot see.
const BAND_LOW: f32 = 52.0;
const BAND_HIGH: f32 = 72.0;

const FEED_ACCEL: f32 = 130.0;
const BLEED_ACCEL: f32 = 95.0;
const RATE_DRAG: f32 = 2.6;
const MAX_RATE: f32 = 60.0;

const SEAL_SECONDS: f32 = 6.0;

/// The host drops a challenge the frame its `tick` returns an outcome, and
/// never asks that frame for its audio — so sealing has to leave the challenge
/// running for a beat or the ting is raised into a valve that is already gone.
const SEALED_HOLD_SECONDS: f32 = 0.9;

const VENT_SECONDS: f32 = 1.2;
const SEALED_PRESSURE: f32 = (BAND_LOW + BAND_HIGH) * 0.5;

#[derive(Debug, Clone, Copy)]
enum ValvePhase {
    Regulating,
    Holding { remaining: f32 },
    Venting { remaining: f32 },
    Sealed { remaining: f32 },
}

pub struct CoolantValve {
    pressure: f32,
    /// PSI per second, signed.
    rate: f32,
    phase: ValvePhase,
    pending_cue: Option<MinigameAudioCue>,
    feed_key: KeyCode,
}

impl CoolantValve {
    pub fn new(keys: MinigameKeys) -> Self {
        Self {
            pressure: 0.0,
            rate: 0.0,
            phase: ValvePhase::Regulating,
            pending_cue: None,
            feed_key: keys.action,
        }
    }

    fn in_band(&self) -> bool {
        (BAND_LOW..=BAND_HIGH).contains(&self.pressure)
    }

    /// Returns whether the line ruptured.
    fn drive(&mut self, feeding: bool, delta_seconds: f32) -> bool {
        let accel = if feeding { FEED_ACCEL } else { -BLEED_ACCEL };

        self.rate += accel * delta_seconds;
        self.rate -= self.rate * RATE_DRAG * delta_seconds;
        self.rate = self.rate.clamp(-MAX_RATE, MAX_RATE);
        self.pressure += self.rate * delta_seconds;

        if self.pressure <= 0.0 {
            self.pressure = 0.0;
            self.rate = self.rate.max(0.0);
        }

        self.pressure > PRESSURE_MAX
    }

    fn rupture(&mut self) {
        self.pressure = 0.0;
        self.rate = 0.0;
        self.pending_cue = Some(MinigameAudioCue::CoolantVent);
        self.phase = ValvePhase::Venting {
            remaining: VENT_SECONDS,
        };
    }
}

impl MinigameInstance for CoolantValve {
    fn title(&self) -> &'static str {
        "Coolant Pressure Regulator"
    }

    fn instructions(&self) -> &'static str {
        "Consult the repair manual."
    }

    fn status(&self) -> String {
        match self.phase {
            ValvePhase::Regulating => format!("{:.0} PSI. Line unsealed.", self.pressure),
            ValvePhase::Holding { remaining } => {
                format!("{:.0} PSI. Holding — {remaining:.1}s", self.pressure)
            }
            ValvePhase::Venting { .. } => "Over-pressure. Line venting.".to_string(),
            ValvePhase::Sealed { .. } => "Pressure held. Line sealed.".to_string(),
        }
    }

    fn take_audio_cues(&mut self) -> Vec<MinigameAudioCue> {
        self.pending_cue.take().into_iter().collect()
    }

    fn visual_state(&self) -> MinigameVisualState {
        MinigameVisualState::CoolantGauge(CoolantGaugeVisualState {
            fill: (self.pressure / PRESSURE_MAX).clamp(0.0, 1.0),
            sealed: matches!(self.phase, ValvePhase::Sealed { .. }),
        })
    }

    fn tick(&mut self, keys: &ButtonInput<KeyCode>, delta_seconds: f32) -> Option<MinigameOutcome> {
        let feeding = keys.pressed(self.feed_key);

        match self.phase {
            ValvePhase::Regulating => {
                if self.drive(feeding, delta_seconds) {
                    self.rupture();
                } else if self.in_band() {
                    self.phase = ValvePhase::Holding {
                        remaining: SEAL_SECONDS,
                    };
                }
            }
            ValvePhase::Holding { remaining } => {
                if self.drive(feeding, delta_seconds) {
                    self.rupture();
                } else if !self.in_band() {
                    self.phase = ValvePhase::Regulating;
                } else {
                    let remaining = remaining - delta_seconds;
                    self.phase = if remaining <= 0.0 {
                        self.pressure = SEALED_PRESSURE;
                        self.rate = 0.0;
                        self.pending_cue = Some(MinigameAudioCue::CoolantSealed);
                        ValvePhase::Sealed {
                            remaining: SEALED_HOLD_SECONDS,
                        }
                    } else {
                        ValvePhase::Holding { remaining }
                    };
                }
            }
            ValvePhase::Venting { remaining } => {
                let remaining = remaining - delta_seconds;
                self.phase = if remaining <= 0.0 {
                    ValvePhase::Regulating
                } else {
                    ValvePhase::Venting { remaining }
                };
            }
            ValvePhase::Sealed { remaining } => {
                let remaining = remaining - delta_seconds;
                if remaining <= 0.0 {
                    return Some(MinigameOutcome::Success);
                }
                self.phase = ValvePhase::Sealed { remaining };
            }
        }

        None
    }
}
