use bevy::prelude::*;

use super::{
    MinigameAudioCue, MinigameInstance, MinigameOutcome, MinigameVisualState, PipeRunVisualState,
    PipeTileVisual,
};
use crate::minigame_keys::MinigameKeys;
use crate::puzzles::scramble;

pub const PIPE_COLS: usize = 4;
pub const PIPE_ROWS: usize = 2;
pub const PIPE_TILES: usize = PIPE_COLS * PIPE_ROWS;

/// A tile's faces, ordered clockwise so that turning a piece is a bit rotate.
const NORTH: u8 = 1;
const EAST: u8 = 2;
const SOUTH: u8 = 4;
const WEST: u8 = 8;
const FACES: u8 = NORTH | EAST | SOUTH | WEST;

/// The feed enters the first tile's west face and leaves the last one's east.
const INLET_TILE: usize = 0;
const DRAIN_TILE: usize = PIPE_TILES - 1;

/// The host drops a challenge the frame its `tick` returns an outcome, so the
/// finished run is held for a beat or it is never drawn.
const LOCKED_HOLD_SECONDS: f32 = 0.9;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipePiece {
    Straight,
    Elbow,
}

impl PipePiece {
    /// Which faces the piece opens onto before it is turned.
    const fn seated(self) -> u8 {
        match self {
            PipePiece::Straight => WEST | EAST,
            PipePiece::Elbow => WEST | SOUTH,
        }
    }

    /// The piece that opens onto exactly `faces` at some turn, if either does.
    fn opening_onto(faces: u8) -> Option<Self> {
        [PipePiece::Straight, PipePiece::Elbow]
            .into_iter()
            .find(|piece| (0..4).any(|turns| turn(piece.seated(), turns) == faces))
    }
}

/// Turns a set of faces `quarters` quarter-turns clockwise.
const fn turn(faces: u8, quarters: u8) -> u8 {
    let mut turned = faces & FACES;
    let mut done = 0;

    while done < quarters % 4 {
        turned = ((turned << 1) | (turned >> 3)) & FACES;
        done += 1;
    }

    turned
}

const fn facing(face: u8) -> u8 {
    turn(face, 2)
}

/// The tile `face` leads to, or `None` at the edge of the grid.
fn neighbour(index: usize, face: u8) -> Option<usize> {
    let row = index / PIPE_COLS;
    let column = index % PIPE_COLS;

    match face {
        NORTH if row > 0 => Some(index - PIPE_COLS),
        SOUTH if row + 1 < PIPE_ROWS => Some(index + PIPE_COLS),
        WEST if column > 0 => Some(index - 1),
        EAST if column + 1 < PIPE_COLS => Some(index + 1),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Coupling {
    piece: PipePiece,
    /// Quarter turns clockwise off the piece's seated position.
    quarters: u8,
}

impl Coupling {
    fn openings(self) -> u8 {
        turn(self.piece.seated(), self.quarters)
    }

    fn turned(self) -> Self {
        Self {
            quarters: (self.quarters + 1) % 4,
            ..self
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum RunPhase {
    Turning,
    Locked { remaining: f32 },
}

pub struct PipeFlow {
    couplings: [Coupling; PIPE_TILES],
    /// Which coupling the wrench is on. Moves along the grid in reading order,
    /// so one pair of keys reaches every tile.
    cursor: usize,
    phase: RunPhase,
    pending_cue: Option<MinigameAudioCue>,
    /// Walks the wrench back a coupling.
    prev_key: KeyCode,
    /// Walks the wrench on a coupling.
    next_key: KeyCode,
    /// Turns the coupling under the wrench a quarter turn.
    turn_key: KeyCode,
}

impl PipeFlow {
    pub fn new(keys: MinigameKeys) -> Self {
        Self::from_seed(rand::random(), keys)
    }

    pub fn from_seed(seed: u64, keys: MinigameKeys) -> Self {
        Self::over(deal(seed).board, keys)
    }

    /// A run sitting over an already-dealt board, with the wrench back at the
    /// inlet end. What both the deal and its tests build through.
    fn over(couplings: [Coupling; PIPE_TILES], keys: MinigameKeys) -> Self {
        Self {
            couplings,
            cursor: 0,
            phase: RunPhase::Turning,
            pending_cue: None,
            prev_key: keys.primary,
            next_key: keys.secondary,
            turn_key: keys.action,
        }
    }

    /// Follows the feed in from the inlet, reporting which couplings it fills
    /// and whether it makes the drain. Two openings per piece means there is
    /// never a choice of where to go next, so this is a single thread.
    fn trace(&self) -> ([bool; PIPE_TILES], bool) {
        let mut wet = [false; PIPE_TILES];
        let mut index = INLET_TILE;
        let mut entering = WEST;

        for _ in 0..PIPE_TILES {
            let openings = self.couplings[index].openings();
            if openings & entering == 0 {
                return (wet, false);
            }

            wet[index] = true;
            let leaving = openings & !entering;

            let Some(next) = neighbour(index, leaving) else {
                // Off the grid: every edge but the drain is a capped stub.
                return (wet, index == DRAIN_TILE && leaving == EAST);
            };

            if wet[next] {
                return (wet, false);
            }

            index = next;
            entering = facing(leaving);
        }

        (wet, false)
    }

    fn flowing(&self) -> bool {
        self.trace().1
    }
}

/// Stands in for a room's dealt keys wherever a `PipeFlow` is built only to
/// read its board back, not to be played — the keys never come into it.
const DUMMY_KEYS: MinigameKeys = MinigameKeys {
    primary: KeyCode::KeyA,
    secondary: KeyCode::KeyD,
    up: KeyCode::KeyW,
    down: KeyCode::KeyS,
    action: KeyCode::Space,
};

struct Deal {
    /// The couplings as the player meets them.
    board: [Coupling; PIPE_TILES],
    /// The same board with the path laid true. Only the tests read it — the
    /// deal computes it anyway on the way to scrambling it, and handing it over
    /// is cheaper than having them search for a solution.
    #[cfg_attr(not(test), allow(dead_code))]
    made_up: [Coupling; PIPE_TILES],
}

/// Deals one run's board: a path is drawn from the inlet to the drain and the
/// couplings on it chosen to fit, so there is always an answer. Everything is
/// then turned at random, which is what leaves the player a job.
fn deal(seed: u64) -> Deal {
    let mut bits = scramble(seed);
    let mut roll = |sides: u64| {
        bits = scramble(bits);
        (bits % sides) as usize
    };

    // Where the run drops a deck is what changes between games.
    let drop_column = roll(PIPE_COLS as u64);
    let path: Vec<usize> = (0..=drop_column)
        .chain((drop_column..PIPE_COLS).map(|column| column + PIPE_COLS))
        .collect();

    // Junk pipe everywhere first; the path is then laid over the top of it.
    let mut board = [(); PIPE_TILES].map(|()| Coupling {
        piece: if roll(2) == 0 {
            PipePiece::Straight
        } else {
            PipePiece::Elbow
        },
        quarters: roll(4) as u8,
    });
    let mut made_up = board;

    for (step, &index) in path.iter().enumerate() {
        // The face the feed arrives on, and the one it leaves by. The ends of
        // the path are fed by the inlet and empty into the drain.
        let entering = match step.checked_sub(1).and_then(|before| path.get(before)) {
            Some(&previous) => facing(toward(previous, index)),
            None => WEST,
        };
        let leaving = match path.get(step + 1) {
            Some(&next) => toward(index, next),
            None => EAST,
        };

        let openings = entering | leaving;
        let piece = PipePiece::opening_onto(openings)
            .expect("a path turns by a right angle at most, which both pieces cover");
        let seated = (0..4)
            .find(|&quarters| turn(piece.seated(), quarters) == openings)
            .expect("the piece was chosen because some turn of it opens onto these faces");

        made_up[index] = Coupling {
            piece,
            quarters: seated,
        };
        board[index] = Coupling {
            piece,
            quarters: (seated + roll(4) as u8) % 4,
        };
    }

    // A run dealt already made up is not a puzzle. One turn always breaks it:
    // both pieces open onto different faces at every quarter.
    if PipeFlow::over(board, DUMMY_KEYS).flowing() {
        board[path[0]] = board[path[0]].turned();
    }

    Deal { board, made_up }
}

/// The face of `from` that its neighbour `to` lies on.
fn toward(from: usize, to: usize) -> u8 {
    [NORTH, EAST, SOUTH, WEST]
        .into_iter()
        .find(|&face| neighbour(from, face) == Some(to))
        .expect("the path only ever steps between neighbouring tiles")
}

impl MinigameInstance for PipeFlow {
    fn title(&self) -> &'static str {
        "Feed Line Coupling"
    }

    fn instructions(&self) -> &'static str {
        "Consult the repair manual."
    }

    fn status(&self) -> String {
        let (wet, drained) = self.trace();
        if drained {
            return "Line made up. Feed running.".to_string();
        }

        let filled = wet.iter().filter(|wet| **wet).count();
        format!("Feed stalled at coupling {}/{PIPE_TILES}.", filled.max(1))
    }

    fn take_audio_cues(&mut self) -> Vec<MinigameAudioCue> {
        self.pending_cue.take().into_iter().collect()
    }

    fn visual_state(&self) -> MinigameVisualState {
        let (wet, drained) = self.trace();

        MinigameVisualState::PipeRun(PipeRunVisualState {
            tiles: std::array::from_fn(|index| PipeTileVisual {
                piece: self.couplings[index].piece,
                quarters: self.couplings[index].quarters,
                flowing: wet[index],
            }),
            cursor: self.cursor,
            drained,
        })
    }

    fn tick(&mut self, keys: &ButtonInput<KeyCode>, delta_seconds: f32) -> Option<MinigameOutcome> {
        match self.phase {
            RunPhase::Turning => {
                if keys.just_pressed(self.prev_key) {
                    self.cursor = (self.cursor + PIPE_TILES - 1) % PIPE_TILES;
                }
                if keys.just_pressed(self.next_key) {
                    self.cursor = (self.cursor + 1) % PIPE_TILES;
                }

                if keys.just_pressed(self.turn_key) {
                    self.couplings[self.cursor] = self.couplings[self.cursor].turned();

                    if self.flowing() {
                        self.pending_cue = Some(MinigameAudioCue::PipesMadeUp);
                        self.phase = RunPhase::Locked {
                            remaining: LOCKED_HOLD_SECONDS,
                        };
                    }
                }
            }
            RunPhase::Locked { remaining } => {
                let remaining = remaining - delta_seconds;
                if remaining <= 0.0 {
                    return Some(MinigameOutcome::Success);
                }
                self.phase = RunPhase::Locked { remaining };
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEEDS: u64 = 500;

    fn press(key: KeyCode) -> ButtonInput<KeyCode> {
        let mut keys = ButtonInput::default();
        keys.press(key);
        keys
    }

    /// The trace walks a single thread on the strength of this — a piece with
    /// three openings would give it a choice it has no way to make.
    #[test]
    fn every_piece_has_two_openings_however_it_is_turned() {
        for piece in [PipePiece::Straight, PipePiece::Elbow] {
            for quarters in 0..4 {
                let openings = turn(piece.seated(), quarters);

                assert_eq!(
                    openings.count_ones(),
                    2,
                    "{piece:?} at {quarters} quarter turns opens onto {openings:04b}"
                );
            }
        }
    }

    /// The point of dealing a path first. Pieces are only ever turned, never
    /// swapped, so the board and its answer must hold the same pieces.
    #[test]
    fn every_board_dealt_can_be_made_up() {
        for seed in 0..SEEDS {
            let dealt = deal(seed);

            assert!(
                PipeFlow::over(dealt.made_up, DUMMY_KEYS).flowing(),
                "seed {seed} dealt a run whose own answer does not carry the feed"
            );

            for (index, (board, made_up)) in dealt.board.iter().zip(&dealt.made_up).enumerate() {
                assert_eq!(
                    board.piece, made_up.piece,
                    "seed {seed} needs coupling {index} swapped, not turned"
                );
            }
        }
    }

    /// And the answer is reachable in play: turning the couplings under the
    /// wrench, with the keys the player actually has, makes the run up.
    #[test]
    fn the_answer_can_be_turned_in_with_the_wrench() {
        for seed in 0..64 {
            let dealt = deal(seed);
            let mut run = PipeFlow::from_seed(seed, DUMMY_KEYS);

            for index in 0..PIPE_TILES {
                run.cursor = index;

                // Bounded at the four positions a coupling has, not run until
                // it matches: the run stops taking turns the moment it is made
                // up, so an unbounded loop would spin on the tiles after that.
                for _ in 0..4 {
                    if run.couplings[index].quarters == dealt.made_up[index].quarters {
                        break;
                    }
                    run.tick(&press(KeyCode::Space), 0.0);
                }
            }

            assert!(
                run.flowing(),
                "seed {seed} could not be worked to its answer"
            );
        }
    }

    /// A board that arrives already finished would be a job the player never
    /// gets to do — and would hand them a free repair.
    #[test]
    fn no_board_is_dealt_already_made_up() {
        for seed in 0..SEEDS {
            assert!(
                !PipeFlow::from_seed(seed, DUMMY_KEYS).flowing(),
                "seed {seed} dealt a run that was already made up"
            );
        }
    }

    /// A and D reach every coupling, and wrap rather than sticking at the ends.
    #[test]
    fn the_wrench_reaches_every_coupling() {
        let mut run = PipeFlow::from_seed(7, DUMMY_KEYS);
        let mut seen = [false; PIPE_TILES];

        for _ in 0..PIPE_TILES {
            seen[run.cursor] = true;
            run.tick(&press(KeyCode::KeyD), 0.0);
        }

        assert!(
            seen.iter().all(|seen| *seen),
            "D does not reach every coupling"
        );
        assert_eq!(run.cursor, 0, "D does not wrap round to the start");

        run.tick(&press(KeyCode::KeyA), 0.0);
        assert_eq!(
            run.cursor,
            PIPE_TILES - 1,
            "A does not wrap back off the start"
        );
    }

    /// Making the run up does not sign it off on the spot: the host drops the
    /// challenge on the frame `tick` returns, so it is held first.
    #[test]
    fn a_made_up_run_is_held_before_it_is_signed_off() {
        const SEED: u64 = 11;

        // Set up one turn short of the answer, so the last press is the one
        // that makes the run up.
        let mut run = PipeFlow::over(deal(SEED).made_up, DUMMY_KEYS);
        run.couplings[INLET_TILE] = run.couplings[INLET_TILE].turned();
        run.cursor = INLET_TILE;
        assert!(!run.flowing(), "the board was set up already made up");

        // Back round to where it was laid: three more turns of the four.
        for _ in 0..3 {
            run.tick(&press(KeyCode::Space), 0.0);
        }

        assert!(run.flowing(), "the board never came good");
        assert!(matches!(run.phase, RunPhase::Locked { .. }));
        assert_eq!(
            run.take_audio_cues(),
            vec![MinigameAudioCue::PipesMadeUp],
            "making the run up raised no cue"
        );
        assert!(
            run.tick(&ButtonInput::default(), LOCKED_HOLD_SECONDS / 2.0)
                .is_none(),
            "the run was signed off before it had been held"
        );
        assert_eq!(
            run.tick(&ButtonInput::default(), LOCKED_HOLD_SECONDS),
            Some(MinigameOutcome::Success)
        );
    }
}
