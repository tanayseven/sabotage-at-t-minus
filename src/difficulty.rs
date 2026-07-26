//! How big a run is: the number of decks the rocket is dealt, chosen on the
//! options screen and read at the start of the next run.
//!
//! Each deck cuts into two rooms, port and starboard, so the deck count is
//! also what decides the room count — and, downstream of that, how many
//! breaches and how much of the manual's room index a run carries.

/// One of five tiers, each worth a fixed number of decks. Ordered so a
/// `step` moves through them in order rather than by name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Difficulty {
    VeryEasy,
    Easy,
    #[default]
    Medium,
    Hard,
    VeryHard,
}

impl Difficulty {
    pub const ALL: [Self; 5] = [
        Self::VeryEasy,
        Self::Easy,
        Self::Medium,
        Self::Hard,
        Self::VeryHard,
    ];

    /// How many decks the rocket has at this tier. Two rooms per deck, so the
    /// room count a run is built with is always double this.
    pub const fn deck_count(self) -> usize {
        match self {
            Self::VeryEasy => 2,
            Self::Easy => 3,
            Self::Medium => 4,
            Self::Hard => 5,
            Self::VeryHard => 6,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::VeryEasy => "Very Easy",
            Self::Easy => "Easy",
            Self::Medium => "Medium",
            Self::Hard => "Hard",
            Self::VeryHard => "Very Hard",
        }
    }

    fn index(self) -> usize {
        Self::ALL
            .iter()
            .position(|tier| *tier == self)
            .expect("every Difficulty is in ALL")
    }

    /// Steps `by` tiers up or down. Clamped rather than wrapping at either
    /// end: a stepper that wrapped Very Hard back to Very Easy would read as
    /// having lost track of which end you were at.
    pub fn step(self, by: isize) -> Self {
        let last = Self::ALL.len() as isize - 1;
        let index = (self.index() as isize + by).clamp(0, last) as usize;
        Self::ALL[index]
    }
}

/// The most decks any tier deals — what the manual's room index page has to
/// have room for, whichever difficulty a run turns out to be.
pub const MAX_DECK_COUNT: usize = Difficulty::VeryHard.deck_count();
/// The fewest decks any tier deals — what has to be enough rooms for every
/// kind of challenge to fit, whichever difficulty a run turns out to be.
pub const MIN_DECK_COUNT: usize = Difficulty::VeryEasy.deck_count();

#[cfg(test)]
mod tests {
    use super::{Difficulty, MAX_DECK_COUNT};

    #[test]
    fn defaults_to_medium() {
        assert_eq!(Difficulty::default(), Difficulty::Medium);
    }

    #[test]
    fn deck_counts_rise_with_each_tier() {
        let counts: Vec<usize> = Difficulty::ALL
            .iter()
            .map(|tier| tier.deck_count())
            .collect();
        let mut sorted = counts.clone();
        sorted.sort_unstable();

        assert_eq!(counts, sorted, "the tiers are not in ascending order");
        assert_eq!(
            counts
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len(),
            counts.len(),
            "two tiers share a deck count"
        );
        assert_eq!(*counts.last().unwrap(), MAX_DECK_COUNT);
    }

    #[test]
    fn stepping_clamps_at_either_end() {
        assert_eq!(Difficulty::VeryEasy.step(-1), Difficulty::VeryEasy);
        assert_eq!(Difficulty::VeryHard.step(1), Difficulty::VeryHard);
        assert_eq!(Difficulty::Medium.step(1), Difficulty::Hard);
        assert_eq!(Difficulty::Medium.step(-1), Difficulty::Easy);
    }

    #[test]
    fn stepping_past_either_end_still_lands_on_it() {
        assert_eq!(Difficulty::VeryEasy.step(-99), Difficulty::VeryEasy);
        assert_eq!(Difficulty::VeryEasy.step(99), Difficulty::VeryHard);
    }
}
