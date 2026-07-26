//! Which room of the rocket each of the run's puzzles is installed in.
//!
//! Every room of the rocket has a breach in it, so there is a job wherever the
//! player goes and the airlock waits on all of them. What is dealt fresh at the
//! start of every run is which challenge each breach opens, and which room the
//! isolation panel is bolted in — a player who has run the rocket before still
//! has to read the manual and work what is in front of them rather than walking
//! a route from memory.
//!
//! The breach and the panel are mounted on different stretches of a room's
//! wall, so the room that draws both can be worked rather than blocked.

use bevy::prelude::*;

use crate::level::{ROOM_COUNT, Room};
use crate::minigames::{MINIGAME_COUNT, MinigameId};

/// Fewer rooms than kinds would leave a challenge with nowhere to be installed.
const _: () = assert!(MINIGAME_COUNT <= ROOM_COUNT);

/// The rooms this run's puzzles are in: the panel's room, and which challenge
/// each room's breach opens. A resource rather than a component because the
/// answer is decided before the level is built and outlives the geometry built
/// from it.
///
/// Every room gets a breach, so the deal is no longer about *which* rooms are
/// worked — it is all of them — but about which challenge turns up where, and
/// which room the panel is bolted in. The breach and the panel are mounted on
/// different stretches of a room's wall, so the room that draws both is worked
/// rather than blocked.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RocketPuzzles {
    pub panel_room: Room,
    /// The challenge behind each room's breach, by room index.
    pub room_minigames: [MinigameId; ROOM_COUNT],
}

impl Default for RocketPuzzles {
    fn default() -> Self {
        Self::from_seed(0)
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
    /// Deals one run: the panel's room, and a challenge for every room.
    ///
    /// The challenges are dealt round the rocket from a rolling start rather
    /// than drawn one room at a time. There are more rooms than kinds of
    /// challenge, so something has to repeat; going round in order is what
    /// stops a run from stacking every breach of one kind at one end of the
    /// rocket, and the rolling start is what stops two runs being the same.
    pub fn from_seed(seed: u64) -> Self {
        let bits = scramble(seed);
        let panel_room = Room::from_index((bits % ROOM_COUNT as u64) as usize);

        let first = (scramble(bits) % MINIGAME_COUNT as u64) as usize;
        let room_minigames =
            std::array::from_fn(|room| MinigameId::ALL[(first + room) % MINIGAME_COUNT]);

        Self {
            panel_room,
            room_minigames,
        }
    }

    /// Where the run's breaches stand and which challenge each one opens: one
    /// per room of the rocket.
    pub fn portal_placements(&self) -> [(Vec2, MinigameId); ROOM_COUNT] {
        std::array::from_fn(|room| {
            (
                Room::from_index(room).portal_mount(),
                self.room_minigames[room],
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The point of the change: there is a job in every room, so a player who
    /// walks into any room of the rocket has something to work there.
    #[test]
    fn every_room_has_a_breach() {
        let puzzles = RocketPuzzles::from_seed(7);
        let placements = puzzles.portal_placements();

        assert_eq!(placements.len(), ROOM_COUNT);

        for index in 0..ROOM_COUNT {
            let room = Room::from_index(index);

            assert!(
                placements.iter().any(|(at, _)| *at == room.portal_mount()),
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
            let puzzles = RocketPuzzles::from_seed(seed);

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
            let puzzles = RocketPuzzles::from_seed(seed);

            for pair in puzzles.room_minigames.windows(2) {
                assert_ne!(
                    pair[0], pair[1],
                    "seed {seed} put the same challenge in two rooms running"
                );
            }
        }
    }

    /// The room that draws the panel as well as a breach is worked breach
    /// first: walking to the panel takes the player into the breach, and a
    /// cleared breach despawns and leaves the panel behind it. What that needs
    /// is for the breach to be on the panel's stretch of wall rather than
    /// somewhere a player could reach the panel without meeting it.
    #[test]
    fn the_breach_sharing_the_panel_s_room_is_met_on_the_way_to_it() {
        use crate::config::PLAYER_HEIGHT;
        use crate::portal::PORTAL_RADIUS;

        for index in 0..ROOM_COUNT {
            let room = Room::from_index(index);
            let breach = room.portal_mount();
            // Where a player stood at the panel, ready to throw a switch, has
            // their centre.
            let working_the_panel = Vec2::new(room.fixture().x, room.floor() + PLAYER_HEIGHT / 2.0);

            assert!(
                working_the_panel.distance(breach) < PORTAL_RADIUS,
                "the panel in {} can be reached without meeting its breach",
                room.label()
            );
        }
    }

    /// Two runs a moment apart must not be the same run: the seed is the app's
    /// uptime in nanoseconds.
    #[test]
    fn seeds_a_moment_apart_deal_different_runs() {
        let seed = 1_234_567_890_u64;
        let differ = (1..=8)
            .filter(|step| RocketPuzzles::from_seed(seed + step) != RocketPuzzles::from_seed(seed));

        assert!(differ.count() >= 6, "the deal barely moves between seeds");
    }
}
