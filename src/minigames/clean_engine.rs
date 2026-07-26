use bevy::prelude::*;

use super::{MinigameInstance, MinigameOutcome};

/// How dirty the engine starts, as a percentage.
const GRIME_START: f32 = 100.0;
/// How many good scrubs it takes to go from filthy to clean.
const STROKES_TO_CLEAN: f32 = 14.0;
/// How much grime one good scrub removes. Derived so that exactly
/// `STROKES_TO_CLEAN` strokes bring `GRIME_START` down to zero.
const SCRUB_PER_STROKE: f32 = GRIME_START / STROKES_TO_CLEAN;

/// Which way the last scrub went. Cleaning is a back-and-forth motion, so we
/// remember the side we last wiped to insist the next one is the *other* side.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Side {
    Left,
    Right,
}

/// The state of one engine-cleaning attempt: how much grime is left, and which
/// way we last wiped.
pub struct CleanEngine {
    grime: f32,
    /// `None` until the very first wipe, since there is no previous side yet.
    last: Option<Side>,
}

impl CleanEngine {
    pub fn new() -> Self {
        Self {
            grime: GRIME_START,
            last: None,
        }
    }
}

impl MinigameInstance for CleanEngine {
    fn title(&self) -> &'static str {
        "Fouled Engine Bell"
    }

    fn instructions(&self) -> &'static str {
        "Alternate A and D to scrub the engine clean."
    }

    fn status(&self) -> String {
        // An eight-cell bar drawn out of text: '#' for grime still there, '_'
        // for the part that has been scrubbed clean.
        const CELLS: usize = 8;
        let filled = ((self.grime / GRIME_START) * CELLS as f32).ceil() as usize;
        let filled = filled.min(CELLS);

        let bar = "#".repeat(filled) + &"_".repeat(CELLS - filled);
        format!("[{bar}] Grime: {}%", self.grime.round() as i32)
    }

    fn tick(
        &mut self,
        keys: &ButtonInput<KeyCode>,
        _delta_seconds: f32,
    ) -> Option<MinigameOutcome> {
        // Which side, if any, was wiped this frame. `A` is left, `D` is right.
        let pressed = if keys.just_pressed(KeyCode::KeyA) {
            Some(Side::Left)
        } else if keys.just_pressed(KeyCode::KeyD) {
            Some(Side::Right)
        } else {
            None
        };

        if let Some(side) = pressed {
            // A wipe only cleans if it goes the opposite way to the last one.
            // The first wipe has nothing before it, so it always counts.
            let alternated = match self.last {
                None => true,
                Some(previous) => previous != side,
            };

            self.last = Some(side);

            if alternated {
                // `.max(0.0)` stops the meter dipping below empty.
                self.grime = (self.grime - SCRUB_PER_STROKE).max(0.0);
            }
        }

        if self.grime <= 0.0 {
            Some(MinigameOutcome::Success)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One frame's worth of input with a single key freshly pressed.
    fn press(key: KeyCode) -> ButtonInput<KeyCode> {
        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(key);
        keys
    }

    /// Runs one frame with `key` down and reports the outcome, if any.
    fn scrub(game: &mut CleanEngine, key: KeyCode) -> Option<MinigameOutcome> {
        game.tick(&press(key), 1.0 / 60.0)
    }

    #[test]
    fn starts_filthy_and_unfinished() {
        let mut game = CleanEngine::new();
        assert_eq!(game.grime, GRIME_START);
        assert_eq!(game.tick(&ButtonInput::default(), 0.0), None);
    }

    #[test]
    fn alternating_wipes_remove_grime() {
        let mut game = CleanEngine::new();
        let before = game.grime;

        scrub(&mut game, KeyCode::KeyA);
        scrub(&mut game, KeyCode::KeyD);

        assert!(game.grime < before);
    }

    #[test]
    fn wiping_the_same_side_twice_only_counts_once() {
        let mut game = CleanEngine::new();

        scrub(&mut game, KeyCode::KeyA);
        let after_first = game.grime;
        scrub(&mut game, KeyCode::KeyA);

        assert_eq!(game.grime, after_first);
    }

    #[test]
    fn enough_alternating_wipes_finishes_the_job() {
        let mut game = CleanEngine::new();
        let mut key = KeyCode::KeyA;
        let mut outcome = None;

        // Plenty of strokes, alternating each time; bail out the moment it's done.
        for _ in 0..100 {
            outcome = scrub(&mut game, key);
            key = if key == KeyCode::KeyA {
                KeyCode::KeyD
            } else {
                KeyCode::KeyA
            };
            if outcome.is_some() {
                break;
            }
        }

        assert_eq!(outcome, Some(MinigameOutcome::Success));
        assert!(game.grime >= 0.0, "the meter fell below empty");
    }
}
