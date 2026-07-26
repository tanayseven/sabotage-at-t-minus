//! Which challenge each of the rocket's rooms is dealt.
//!
//! Every room of the rocket has a breach in it, so there is a job wherever the
//! player goes and the airlock waits on all of them. What is dealt fresh at the
//! start of every run is which challenge each breach opens — a player who has
//! run the rocket before still has to read the manual and work what is in
//! front of them rather than walking a route from memory.
//!
//! Every room also carries its own isolation panel — see
//! [`crate::panel::Panels`] — mounted on a different stretch of wall from the
//! breach, so the room that draws both can be worked rather than blocked.

use bevy::prelude::*;

use crate::difficulty::MIN_DECK_COUNT;
use crate::level::{ROOMS_PER_DECK, Room};
use crate::minigames::{MINIGAME_COUNT, MinigameId};

/// Fewer rooms than kinds would leave a challenge with nowhere to be
/// installed — checked against the smallest difficulty deals, since that is
/// the tightest a run ever gets.
const _: () = assert!(MINIGAME_COUNT <= MIN_DECK_COUNT * ROOMS_PER_DECK);

/// Which challenge each room's breach opens this run. A resource rather than a
/// component because the answer is decided before the level is built and
/// outlives the geometry built from it.
///
/// Sized to the run's room count rather than a fixed one, since how many decks
/// the rocket has is picked per run.
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct RocketPuzzles {
    /// The challenge behind each room's breach, by room index.
    pub room_minigames: Vec<MinigameId>,
}

impl Default for RocketPuzzles {
    /// A placeholder dealt over Medium's room count — every run overwrites
    /// this before the level it names is ever built, so what it is dealt off
    /// only matters for satisfying the resource's existence at startup.
    fn default() -> Self {
        Self::from_seed(0, 4 * ROOMS_PER_DECK)
    }
}

/// Spreads a seed over all 64 bits, so seeds a fraction of a second apart deal
/// unrelated rooms instead of walking along the list. (splitmix64's finaliser.)
pub const fn scramble(seed: u64) -> u64 {
    let mut z = seed.wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

impl RocketPuzzles {
    /// Deals one run: a challenge for every room.
    ///
    /// The challenges are dealt round the rocket from a rolling start rather
    /// than drawn one room at a time. There are more rooms than kinds of
    /// challenge, so something has to repeat; going round in order is what
    /// stops a run from stacking every breach of one kind at one end of the
    /// rocket, and the rolling start is what stops two runs being the same.
    pub fn from_seed(seed: u64, room_count: usize) -> Self {
        let bits = scramble(seed);

        let first = (scramble(bits) % MINIGAME_COUNT as u64) as usize;
        let room_minigames: Vec<MinigameId> = (0..room_count)
            .map(|room| MinigameId::ALL[(first + room) % MINIGAME_COUNT])
            .collect();

        Self { room_minigames }
    }

    /// Where the run's breaches stand and which challenge each one opens: one
    /// per room of the rocket.
    pub fn portal_placements(&self) -> Vec<(Room, Vec2, MinigameId)> {
        self.room_minigames
            .iter()
            .enumerate()
            .map(|(room, minigame)| {
                let room = Room::from_index(room);
                (room, room.portal_mount(), *minigame)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_ROOM_COUNT: usize = 4 * ROOMS_PER_DECK;

    /// The point of the change: there is a job in every room, so a player who
    /// walks into any room of the rocket has something to work there.
    #[test]
    fn every_room_has_a_breach() {
        let puzzles = RocketPuzzles::from_seed(7, TEST_ROOM_COUNT);
        let placements = puzzles.portal_placements();

        assert_eq!(placements.len(), TEST_ROOM_COUNT);

        for index in 0..TEST_ROOM_COUNT {
            let room = Room::from_index(index);

            assert!(
                placements
                    .iter()
                    .any(|(_, at, _)| *at == room.portal_mount()),
                "{} has no breach in it",
                room.label()
            );
        }
    }

    /// A run that never installed one of the challenges would be a run with a
    /// page of the manual that never comes up.
    #[test]
    fn every_challenge_is_installed_somewhere() {
        for seed in 0..500u64 {
            let puzzles = RocketPuzzles::from_seed(seed, TEST_ROOM_COUNT);

            for minigame in MinigameId::ALL {
                assert!(
                    puzzles.room_minigames.contains(&minigame),
                    "seed {seed} installs {minigame:?} nowhere"
                );
            }
        }
    }

    /// Dealing round the rocket rather than drawing each room on its own is
    /// what spreads the kinds out: no two rooms in a row open the same
    /// challenge while there is more than one kind to hand.
    #[test]
    fn neighbouring_rooms_open_different_challenges() {
        for seed in 0..500u64 {
            let puzzles = RocketPuzzles::from_seed(seed, TEST_ROOM_COUNT);

            for pair in puzzles.room_minigames.windows(2) {
                assert_ne!(
                    pair[0], pair[1],
                    "seed {seed} put the same challenge in two rooms running"
                );
            }
        }
    }

    /// Every room now carries both a panel and a breach, mounted on different
    /// stretches of its wall — working one must never be blocked by standing
    /// in the other.
    ///
    /// Except in the bottom deck's port room: see
    /// [`crate::level::Room::panel_mount`] for why that one room's panel is
    /// mounted alongside its breach instead.
    #[test]
    fn every_room_s_panel_and_breach_stand_clear_of_each_other() {
        use crate::config::PLAYER_HEIGHT;
        use crate::level::Side;
        use crate::portal::PORTAL_RADIUS;

        for index in 0..TEST_ROOM_COUNT {
            let room = Room::from_index(index);
            if room.deck == 0 && room.side == Side::Port {
                continue;
            }

            let breach = room.portal_mount();
            // Where a player stood at the panel, ready to throw a switch, has
            // their centre.
            let working_the_panel =
                Vec2::new(room.panel_mount().x, room.floor() + PLAYER_HEIGHT / 2.0);

            assert!(
                working_the_panel.distance(breach) > PORTAL_RADIUS,
                "the panel in {} cannot be worked without also standing in its breach",
                room.label()
            );
        }
    }

    /// There are only as many ways to rotate the challenges as there are kinds
    /// of challenge, so the deal alone repeats far sooner than a run actually
    /// does — keeping two nearby seeds from feeling like the same run is
    /// [`crate::panel::Panels`]'s job, not this one's. What this deal still has
    /// to do is use every rotation there is, rather than favouring one.
    #[test]
    fn every_rotation_of_the_challenges_turns_up() {
        for minigame in MinigameId::ALL {
            assert!(
                (0..64u64).any(|seed| RocketPuzzles::from_seed(seed, TEST_ROOM_COUNT)
                    .room_minigames[0]
                    == minigame),
                "{minigame:?} never opens the first room"
            );
        }
    }
}
