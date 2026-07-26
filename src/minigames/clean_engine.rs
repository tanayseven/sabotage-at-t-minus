//! Scrubbing the soot out of the engine bell.
//!
//! The bore is worked as a grid: a stack of courses running around it, each of
//! which has to be cut clean before the bell will pass. Two hands do two
//! different jobs — W and S walk the brush up and down the courses, A and D cut
//! along the course it is standing on — and neither is any use without the
//! other. That split is the point of the challenge: everything else in the game
//! is worked with one control, and this is the one that asks the player to hold
//! a position *and* a rhythm at the same time.
//!
//! The brush is set down in the middle of the bore rather than at one end, so
//! the job opens as a choice of which way to work rather than as a list to run
//! down from the top. That is also why there are an odd number of courses:
//! there has to *be* a middle to start in.
//!
//! The rhythm rule is that a stroke only counts if it goes back the other way.
//! Leaning on one key does nothing after the first press, so the player has to
//! work A and D against each other rather than mash a single side — which is
//! what makes it read as scrubbing rather than as a button count.
//!
//! There is no way to lose in here. The bore never re-fouls and the challenge
//! never returns a failure: the only pressure is the launch clock running
//! outside, and every second spent scrubbing is a second not spent on the rest
//! of the rocket. So `tick` returns either nothing or a sign-off, never a loss.

use bevy::prelude::*;

use super::{
    BORE_CELLS, BORE_ROWS, EngineBoreVisualState, MinigameInstance, MinigameOutcome,
    MinigameVisualState,
};

/// Courses stacked up the bore — the rows of the grid. Taken from the art
/// rather than set here: the grid is painted into both plates, so a second
/// opinion about how many courses there are would draw a bore the challenge
/// disagreed with. Odd, so that the brush has a middle course to be set down on
/// with as much bore above it as below.
const ROWS: usize = BORE_ROWS;

/// The course the brush is set down on: the middle one. Integer division lands
/// on it exactly because [`ROWS`] is odd — two courses above, two below.
const STARTING_ROW: usize = ROWS / 2;

/// Odd, so there is a middle course to start on at all — an even stack would
/// put the brush half a course out and quietly favour one end of the bore. And
/// at least three, so there is bore on both sides of it: with fewer, starting
/// in the middle is just starting at an end by another name.
const _: () = assert!(ROWS >= 3 && ROWS % 2 == 1);

/// How far along a course one stroke cuts — the columns of the grid, painted
/// into the plates the same way [`ROWS`] is. `ROWS` by this is the whole job:
/// enough strokes to feel like work, short enough that it is not the whole
/// two-minute run.
const CELLS_PER_ROW: u32 = BORE_CELLS;

/// Which way the brush was last pulled. There is no "neither" once a course is
/// under way, so the starting state is an [`Option`] rather than a variant —
/// with the brush not yet moved, *either* side is a valid opening.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Side {
    Left,
    Right,
}

impl Side {
    fn opposite(self) -> Self {
        match self {
            Side::Left => Side::Right,
            Side::Right => Side::Left,
        }
    }
}

pub struct CleanEngine {
    /// Cells cut clean in each course, counted from the left. Cell *counts*
    /// rather than a grid of flags: a course is always cut left to right, so
    /// how far along the brush has got says everything about it, and the last
    /// stroke lands exactly on clean instead of leaving a hair of rounding
    /// behind and asking for one more.
    rows: [u32; ROWS],
    /// The course the brush is standing on.
    row: usize,
    /// The side the brush was last pulled to on this course.
    last: Option<Side>,
}

impl CleanEngine {
    pub fn new() -> Self {
        Self {
            rows: [0; ROWS],
            row: STARTING_ROW,
            last: None,
        }
    }

    fn cleaned(&self) -> bool {
        self.rows.iter().all(|cut| *cut >= CELLS_PER_ROW)
    }

    /// How much of the whole bore has been cut, 0 to 1.
    fn cut_fraction(&self) -> f32 {
        let cut: u32 = self.rows.iter().map(|cut| (*cut).min(CELLS_PER_ROW)).sum();

        cut as f32 / (ROWS as u32 * CELLS_PER_ROW) as f32
    }

    /// Walks the brush `by` courses. Clamped rather than wrapping: the bore has
    /// a top and a bottom, and running off one onto the other would lose the
    /// player their place in a display they are reading as a picture.
    ///
    /// Moving lifts the brush, which is why `last` is dropped — coming onto a
    /// fresh course with the wrong hand loaded would eat the player's first
    /// press for no reason they could see.
    fn walk(&mut self, by: isize) {
        let moved = self.row.saturating_add_signed(by).min(ROWS - 1);

        if moved != self.row {
            self.row = moved;
            self.last = None;
        }
    }

    /// Takes a pull of the brush to `side`, and says whether it cut anything.
    /// Only the stroke back the other way cuts; the first one on a course
    /// always does, since with the brush just set down either side is a way
    /// back. A course already clean takes nothing, however it is worked.
    fn pull(&mut self, side: Side) -> bool {
        if self.rows[self.row] >= CELLS_PER_ROW {
            return false;
        }
        if !self.last.is_none_or(|last| last.opposite() == side) {
            return false;
        }

        self.last = Some(side);
        self.rows[self.row] += 1;
        true
    }

    /// The side asked for this frame, if exactly one was. Both keys at once is
    /// deliberately nothing: with two hands down there is no telling which way
    /// the brush went, and letting it count would hand the player a way to
    /// scrub twice as fast by pressing both together instead of alternating.
    fn asked_for(keys: &ButtonInput<KeyCode>) -> Option<Side> {
        match (
            keys.just_pressed(KeyCode::KeyA),
            keys.just_pressed(KeyCode::KeyD),
        ) {
            (true, false) => Some(Side::Left),
            (false, true) => Some(Side::Right),
            _ => None,
        }
    }

    /// How far up or down the bore the brush was asked to go this frame. Both
    /// keys at once cancel for the same reason both hands do.
    fn walked(keys: &ButtonInput<KeyCode>) -> isize {
        match (
            keys.just_pressed(KeyCode::KeyW),
            keys.just_pressed(KeyCode::KeyS),
        ) {
            (true, false) => -1,
            (false, true) => 1,
            _ => 0,
        }
    }
}

impl MinigameInstance for CleanEngine {
    fn title(&self) -> &'static str {
        "Fouled Engine Bell"
    }

    fn instructions(&self) -> &'static str {
        "Consult the repair manual."
    }

    /// One line under the picture. The grid itself is drawn from
    /// [`Self::visual_state`], so this says how the job stands rather than
    /// spelling the bore out a second time — and it is kept short because the
    /// window gives the status about twenty characters before it wraps.
    fn status(&self) -> String {
        format!("Bore {:.0}% clean.", self.cut_fraction() * 100.0)
    }

    fn visual_state(&self) -> MinigameVisualState {
        MinigameVisualState::EngineBore(EngineBoreVisualState {
            cut: self.rows,
            row: self.row,
        })
    }

    fn tick(
        &mut self,
        keys: &ButtonInput<KeyCode>,
        _delta_seconds: f32,
    ) -> Option<MinigameOutcome> {
        // Walking first, so a press that moves the brush and a press that cuts
        // can both land in one frame without the cut going onto the course the
        // player has just left.
        let by = Self::walked(keys);
        if by != 0 {
            self.walk(by);
        }

        if let Some(side) = Self::asked_for(keys) {
            self.pull(side);
        }

        // Nothing else ends this: no timer of its own and no way to foul it
        // back up, so the only outcome it ever hands back is the sign-off.
        self.cleaned().then_some(MinigameOutcome::Success)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A frame's worth of time. Nothing in here reads the clock, but `tick`
    /// still has to be handed one.
    const FRAME: f32 = 1.0 / 60.0;

    /// One frame with `keys` freshly pressed. A new [`ButtonInput`] per call is
    /// what makes each of these a *press* rather than a key still held down
    /// from the frame before — which is the distinction the whole mechanic
    /// turns on.
    fn tick_with(game: &mut CleanEngine, keys: &[KeyCode]) -> Option<MinigameOutcome> {
        let mut input = ButtonInput::<KeyCode>::default();
        for key in keys {
            input.press(*key);
        }

        game.tick(&input, FRAME)
    }

    /// Works the brush properly for `strokes` frames. The side is taken from
    /// where the brush already is rather than counted off from A, so two calls
    /// in a row carry on alternating instead of repeating a side and stalling.
    fn scrub(game: &mut CleanEngine, strokes: u32) -> Option<MinigameOutcome> {
        let mut outcome = None;

        for _ in 0..strokes {
            let key = match game.last.map_or(Side::Left, Side::opposite) {
                Side::Left => KeyCode::KeyA,
                Side::Right => KeyCode::KeyD,
            };
            outcome = tick_with(game, &[key]);
        }

        outcome
    }

    /// The whole job, worked the way a player would: up to the top of the bore
    /// from where the brush was set down, then cut and drop, cut and drop.
    fn scrub_the_whole_bore(game: &mut CleanEngine) -> Option<MinigameOutcome> {
        let mut outcome = None;

        for _ in 0..ROWS {
            tick_with(game, &[KeyCode::KeyW]);
        }

        for course in 0..ROWS {
            if course > 0 {
                tick_with(game, &[KeyCode::KeyS]);
            }
            outcome = scrub(game, CELLS_PER_ROW);
        }

        outcome
    }

    #[test]
    fn a_fresh_bell_is_filthy_and_unfinished() {
        let mut game = CleanEngine::new();

        assert_eq!(game.rows, [0; ROWS]);
        assert_eq!(tick_with(&mut game, &[]), None, "it signed itself off");
        assert!(game.status().contains("0%"), "{}", game.status());
    }

    /// The brush is set down mid-bore, with as much of the job above it as
    /// below — the player opens by choosing a way to work, not by starting at
    /// the top of a list.
    #[test]
    fn the_brush_starts_in_the_middle_of_the_bore() {
        let game = CleanEngine::new();

        assert_eq!(game.row, STARTING_ROW);
        assert_eq!(
            game.row,
            ROWS - 1 - game.row,
            "the brush is set down off-centre",
        );
    }

    #[test]
    fn alternating_strokes_cut_along_the_course() {
        let mut game = CleanEngine::new();

        scrub(&mut game, 3);

        assert_eq!(game.rows[STARTING_ROW], 3);
        assert_eq!(
            game.rows.iter().filter(|cut| **cut > 0).count(),
            1,
            "it cut other courses too",
        );
    }

    /// The rule the challenge exists for: the brush only cuts on the way back.
    #[test]
    fn leaning_on_one_side_only_counts_once() {
        let mut game = CleanEngine::new();

        for _ in 0..6 {
            tick_with(&mut game, &[KeyCode::KeyA]);
        }

        assert_eq!(
            game.rows[STARTING_ROW], 1,
            "the same side was scrubbed with twice",
        );

        // …and the way back still lands, so a repeat does not lock anything up.
        tick_with(&mut game, &[KeyCode::KeyD]);
        assert_eq!(game.rows[STARTING_ROW], 2);
    }

    /// Both hands down is not a stroke — otherwise pressing A and D together
    /// would scrub twice a frame and beat alternating outright.
    #[test]
    fn both_hands_at_once_is_not_a_stroke() {
        let mut game = CleanEngine::new();

        for _ in 0..5 {
            tick_with(&mut game, &[KeyCode::KeyA, KeyCode::KeyD]);
        }

        assert_eq!(game.rows, [0; ROWS]);
    }

    #[test]
    fn either_side_may_open_a_course() {
        let mut game = CleanEngine::new();
        tick_with(&mut game, &[KeyCode::KeyD]);

        assert_eq!(
            game.rows[STARTING_ROW], 1,
            "the brush would not start on the right",
        );
        assert_eq!(game.last, Some(Side::Right));
    }

    #[test]
    fn w_and_s_walk_the_brush_up_and_down_the_bore() {
        let mut game = CleanEngine::new();

        tick_with(&mut game, &[KeyCode::KeyS]);
        assert_eq!(game.row, STARTING_ROW + 1);

        tick_with(&mut game, &[KeyCode::KeyW]);
        assert_eq!(game.row, STARTING_ROW);

        tick_with(&mut game, &[KeyCode::KeyW]);
        assert_eq!(game.row, STARTING_ROW - 1);
    }

    /// Both ways out of the middle have to be open, or starting there is just a
    /// longer walk to the end the player was going to work from anyway.
    #[test]
    fn the_bore_runs_both_ways_from_where_the_brush_starts() {
        let mut game = CleanEngine::new();

        tick_with(&mut game, &[KeyCode::KeyW]);
        scrub(&mut game, 1);
        tick_with(&mut game, &[KeyCode::KeyS]);
        tick_with(&mut game, &[KeyCode::KeyS]);
        scrub(&mut game, 1);

        assert_eq!(game.rows[STARTING_ROW - 1], 1, "the way up was not worked");
        assert_eq!(
            game.rows[STARTING_ROW + 1],
            1,
            "the way down was not worked"
        );
    }

    /// The bore has a top and a bottom. Running off either would put the brush
    /// somewhere the display does not draw.
    #[test]
    fn the_brush_stops_at_the_ends_of_the_bore() {
        let mut game = CleanEngine::new();

        for _ in 0..ROWS + 3 {
            tick_with(&mut game, &[KeyCode::KeyW]);
        }
        assert_eq!(game.row, 0, "the brush went off the top of the bore");

        for _ in 0..ROWS + 3 {
            tick_with(&mut game, &[KeyCode::KeyS]);
        }
        assert_eq!(game.row, ROWS - 1, "the brush went off the bottom");
    }

    /// Moving lifts the brush, so the next course opens on either hand. Landing
    /// on a fresh course with the wrong hand loaded would swallow a press for
    /// no reason the player could see.
    #[test]
    fn changing_course_lets_either_side_open_again() {
        let mut game = CleanEngine::new();

        tick_with(&mut game, &[KeyCode::KeyA]);
        tick_with(&mut game, &[KeyCode::KeyS]);
        assert_eq!(game.last, None, "the brush carried a side across");

        tick_with(&mut game, &[KeyCode::KeyA]);
        assert_eq!(
            game.rows[STARTING_ROW + 1],
            1,
            "the fresh course would not open on A",
        );
    }

    /// Strokes on a course already cut do nothing — the player has to walk the
    /// brush on rather than keep working a clean one.
    #[test]
    fn a_course_already_cut_takes_nothing_more() {
        let mut game = CleanEngine::new();

        scrub(&mut game, CELLS_PER_ROW + 5);

        assert_eq!(game.rows[STARTING_ROW], CELLS_PER_ROW);
        assert_eq!(
            game.rows.iter().filter(|cut| **cut > 0).count(),
            1,
            "it spilled onto another course",
        );
    }

    /// One course cut is not the job — the sign-off waits for the whole bore.
    #[test]
    fn one_clean_course_does_not_sign_the_job_off() {
        let mut game = CleanEngine::new();

        assert_eq!(scrub(&mut game, CELLS_PER_ROW), None);
    }

    #[test]
    fn cutting_every_course_signs_the_job_off() {
        let mut game = CleanEngine::new();

        assert_eq!(
            scrub_the_whole_bore(&mut game),
            Some(MinigameOutcome::Success)
        );
        assert_eq!(game.rows, [CELLS_PER_ROW; ROWS]);
    }

    /// What the pictures are driven off. A snapshot that did not follow the
    /// brush would leave the plates showing a bore nobody is working.
    #[test]
    fn the_picture_follows_the_brush_and_the_cutting() {
        let mut game = CleanEngine::new();
        tick_with(&mut game, &[KeyCode::KeyS]);
        scrub(&mut game, 2);

        let MinigameVisualState::EngineBore(bore) = game.visual_state() else {
            panic!("the bore is not drawn as a bore");
        };

        assert_eq!(bore.row, STARTING_ROW + 1, "the brush is drawn elsewhere");
        assert_eq!(bore.cut[STARTING_ROW + 1], 2, "the cutting is not drawn");
        assert_eq!(bore.cut[STARTING_ROW], 0, "it drew a course it never cut");
    }

    /// The clean plate is let out cell by cell, so a course can never ask for
    /// more of it than there is — a window wider than the plate would show the
    /// bore's own background past the end of the picture.
    #[test]
    fn no_course_is_ever_drawn_cut_past_its_end() {
        let mut game = CleanEngine::new();

        for _ in 0..ROWS {
            scrub(&mut game, CELLS_PER_ROW + 3);
            tick_with(&mut game, &[KeyCode::KeyS]);

            let MinigameVisualState::EngineBore(bore) = game.visual_state() else {
                panic!("the bore is not drawn as a bore");
            };

            assert!(bore.row < ROWS, "the brush is drawn off the plate");
            for cut in bore.cut {
                assert!(cut <= CELLS_PER_ROW, "a course is drawn {cut} cells cut");
            }
        }
    }

    /// What the status line has room for: a 380px window with 22px of padding
    /// either side, set at 28px, comes to about twenty characters. Past that it
    /// wraps under the picture and pushes the layout about.
    const MAX_STATUS_CHARS: usize = 20;

    #[test]
    fn the_status_line_fits_the_window() {
        let mut game = CleanEngine::new();

        for _ in 0..ROWS {
            let status = game.status();

            assert_eq!(status.lines().count(), 1, "the status is not one line");
            assert!(
                status.chars().count() <= MAX_STATUS_CHARS,
                "the status is too wide for the window: {status:?}",
            );

            scrub(&mut game, CELLS_PER_ROW);
            tick_with(&mut game, &[KeyCode::KeyS]);
        }
    }
}
