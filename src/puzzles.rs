//! Which room of the rocket each of the run's puzzles is installed in.
//!
//! The rocket has six rooms and the game has three things to be worked: the
//! isolation panel and one breach per kind of minigame. Every one of them is
//! installed in a room of its own, picked fresh at the start of every run — a
//! player who has run the rocket before still has to find each job rather than
//! walking a route from memory, and no two jobs are ever stacked in the same
//! room where one would be worked through the other.
//!
//! The crossing passes through all six rooms, so dealing three of them out is
//! enough to put every puzzle on the player's way to the airlock without any of
//! them having to be hunted for.

use bevy::prelude::*;

use crate::level::{ROOM_COUNT, Room};
use crate::minigames::{MINIGAME_COUNT, MinigameId};

/// One room for the panel and one for each challenge.
pub const PUZZLE_COUNT: usize = 1 + MINIGAME_COUNT;

/// There has to be a room for each of them, or the deal below would have to
/// double one up — which is the one thing this module exists to prevent.
const _: () = assert!(PUZZLE_COUNT <= ROOM_COUNT);

/// The rooms this run's puzzles are in: the panel's, and one per challenge, all
/// different. A resource rather than a component because the answer is decided
/// before the level is built and outlives the geometry built from it.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RocketPuzzles {
    pub panel_room: Room,
    /// Room per entry of [`MinigameId::ALL`], in that order.
    pub portal_rooms: [Room; MINIGAME_COUNT],
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
    /// Deals the rooms for one run.
    pub fn from_seed(seed: u64) -> Self {
        let dealt = deal_rooms(seed);

        Self {
            panel_room: dealt[0],
            portal_rooms: std::array::from_fn(|index| dealt[1 + index]),
        }
    }

    /// Where the run's breaches stand and which challenge each one opens: one
    /// per room dealt above, in the order the manual documents them.
    pub fn portal_placements(&self) -> [(Vec2, MinigameId); MINIGAME_COUNT] {
        std::array::from_fn(|index| {
            (
                self.portal_rooms[index].portal_mount(),
                MinigameId::ALL[index],
            )
        })
    }
}

/// [`PUZZLE_COUNT`] different rooms, drawn from the whole rocket without
/// replacement — the draw is what says two puzzles never share a room.
fn deal_rooms(seed: u64) -> [Room; PUZZLE_COUNT] {
    let mut pool: [usize; ROOM_COUNT] = std::array::from_fn(|index| index);
    let mut left = ROOM_COUNT;
    let mut bits = scramble(seed);

    std::array::from_fn(|_| {
        bits = scramble(bits);
        let picked = (bits % left as u64) as usize;
        let room = pool[picked];

        // The room just taken is replaced by the last one still in the pool, so
        // the next draw is over what is left rather than over the whole rocket.
        left -= 1;
        pool[picked] = pool[left];

        Room::from_index(room)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The whole point of the deal: nothing is installed on top of anything
    /// else. Two puzzles in one room would put a portal over the panel — and the
    /// portal is walked into rather than worked, so the panel behind it could
    /// not be reached without being pulled into a minigame first.
    #[test]
    fn no_two_puzzles_share_a_room() {
        for seed in 0..2_000u64 {
            let puzzles = RocketPuzzles::from_seed(seed);
            let mut rooms = vec![puzzles.panel_room];
            rooms.extend(puzzles.portal_rooms);

            for (index, room) in rooms.iter().enumerate() {
                assert!(
                    !rooms[index + 1..].contains(room),
                    "seed {seed} put two puzzles in {}",
                    room.label()
                );
            }
        }
    }

    /// Every room has to be one each job can turn up in, or the rocket is
    /// narrower than it looks and the same rooms are worked every run.
    #[test]
    fn every_room_comes_up_for_every_puzzle() {
        let mut seen = [[false; ROOM_COUNT]; PUZZLE_COUNT];

        for seed in 0..2_000u64 {
            let puzzles = RocketPuzzles::from_seed(seed);
            let dealt = [puzzles.panel_room].into_iter().chain(puzzles.portal_rooms);

            for (puzzle, room) in dealt.enumerate() {
                let index = (0..ROOM_COUNT)
                    .position(|index| Room::from_index(index) == room)
                    .expect("dealt a room that is not in the rocket");
                seen[puzzle][index] = true;
            }
        }

        for (puzzle, rooms) in seen.iter().enumerate() {
            assert!(
                rooms.iter().all(|seen| *seen),
                "puzzle {puzzle} is never installed in some room"
            );
        }
    }

    /// Two runs a moment apart must not be the same run: the seed is the app's
    /// uptime in nanoseconds.
    #[test]
    fn seeds_a_moment_apart_deal_different_rooms() {
        let seed = 1_234_567_890_u64;
        let differ = (1..=8)
            .filter(|step| RocketPuzzles::from_seed(seed + step) != RocketPuzzles::from_seed(seed));

        assert!(differ.count() >= 6, "the deal barely moves between seeds");
    }

    /// The breaches stand in the rooms that were dealt, one per kind of
    /// challenge, so a run works every puzzle the game has.
    #[test]
    fn one_portal_of_each_kind_stands_in_its_own_room() {
        let puzzles = RocketPuzzles::from_seed(5);
        let placements = puzzles.portal_placements();

        for minigame in MinigameId::ALL {
            assert!(
                placements.iter().any(|(_, id)| *id == minigame),
                "{minigame:?} is not installed anywhere in the rocket"
            );
        }

        for (index, (at, _)) in placements.iter().enumerate() {
            assert_eq!(*at, puzzles.portal_rooms[index].portal_mount());
        }
    }
}
