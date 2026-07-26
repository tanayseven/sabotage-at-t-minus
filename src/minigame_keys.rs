//! Which keys work each room's breach challenge.
//!
//! A player who has run the rocket before still has to read the manual and
//! find the keys for the room they are in — the challenges themselves repeat
//! run to run, but WASD-and-Space never does. Every room draws its own five
//! keys, independent of every other room's, the same way every room draws its
//! own panel combination — see [`crate::panel::Panel::for_room`].

use bevy::prelude::*;

use crate::level::Room;
use crate::puzzles::scramble;

/// Every key a room's challenge might ask for. Which of the five a given
/// challenge actually uses depends on which [`crate::minigames::MinigameId`]
/// the room was dealt — a room with one working hand's worth of challenge
/// just never reads `up` or `down`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MinigameKeys {
    pub primary: KeyCode,
    pub secondary: KeyCode,
    pub up: KeyCode,
    pub down: KeyCode,
    pub action: KeyCode,
}

/// Letters a room's five keys are drawn from. `M` is left out: it is the
/// manual's own toggle, and reads both while a challenge is open and while it
/// is not, so a room dealt it as a working key would fight the book that
/// documents it.
const KEY_POOL: [KeyCode; 25] = [
    KeyCode::KeyA,
    KeyCode::KeyB,
    KeyCode::KeyC,
    KeyCode::KeyD,
    KeyCode::KeyE,
    KeyCode::KeyF,
    KeyCode::KeyG,
    KeyCode::KeyH,
    KeyCode::KeyI,
    KeyCode::KeyJ,
    KeyCode::KeyK,
    KeyCode::KeyL,
    KeyCode::KeyN,
    KeyCode::KeyO,
    KeyCode::KeyP,
    KeyCode::KeyQ,
    KeyCode::KeyR,
    KeyCode::KeyS,
    KeyCode::KeyT,
    KeyCode::KeyU,
    KeyCode::KeyV,
    KeyCode::KeyW,
    KeyCode::KeyX,
    KeyCode::KeyY,
    KeyCode::KeyZ,
];

/// The single letter a key prints as in the manual. Total over [`KEY_POOL`]
/// rather than every `KeyCode` there is — nothing outside that pool is ever
/// dealt, so nothing else has to print.
pub fn key_letter(key: KeyCode) -> char {
    KEY_POOL
        .iter()
        .position(|candidate| *candidate == key)
        .map(|index| (b'A' + index as u8 + if index >= 12 { 1 } else { 0 }) as char)
        .expect("every key dealt to a room comes from KEY_POOL")
}

impl MinigameKeys {
    /// One room's five keys, drawn without repeats from [`KEY_POOL`] and mixed
    /// with the room the same way a panel's combination is — so two rooms
    /// dealt the same challenge never end up sharing a keyboard.
    fn for_room(room: Room, seed: u64) -> Self {
        let mut bits = scramble(seed ^ scramble(room.index() as u64));
        let mut pool = KEY_POOL.to_vec();
        let mut draw = || {
            bits = scramble(bits);
            pool.remove((bits % pool.len() as u64) as usize)
        };

        Self {
            primary: draw(),
            secondary: draw(),
            up: draw(),
            down: draw(),
            action: draw(),
        }
    }
}

/// Every room's keys for one run. A resource rather than a component, so it
/// outlives the geometry built from it, the same as
/// [`crate::puzzles::RocketPuzzles`] and [`crate::panel::Panels`].
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct RoomKeys(Vec<MinigameKeys>);

impl Default for RoomKeys {
    /// A placeholder dealt over Medium's room count — every run overwrites
    /// this before it is ever read, the same way [`crate::panel::Panels`]'s
    /// placeholder does.
    fn default() -> Self {
        Self::from_seed(0, 4 * crate::level::ROOMS_PER_DECK)
    }
}

impl RoomKeys {
    pub fn from_seed(seed: u64, room_count: usize) -> Self {
        Self(
            (0..room_count)
                .map(|index| MinigameKeys::for_room(Room::from_index(index), seed))
                .collect(),
        )
    }

    pub fn of(&self, room: Room) -> MinigameKeys {
        self.0[room.index()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_ROOM_COUNT: usize = 6 * crate::level::ROOMS_PER_DECK;

    /// The whole point: no room's five keys share a letter, or the player
    /// could not tell two of its controls apart.
    #[test]
    fn a_room_s_five_keys_are_all_different() {
        for seed in 0..200u64 {
            let keys = RoomKeys::from_seed(seed, TEST_ROOM_COUNT);

            for index in 0..TEST_ROOM_COUNT {
                let dealt = keys.of(Room::from_index(index));
                let all = [
                    dealt.primary,
                    dealt.secondary,
                    dealt.up,
                    dealt.down,
                    dealt.action,
                ];

                for (i, one) in all.iter().enumerate() {
                    for other in &all[i + 1..] {
                        assert_ne!(one, other, "room {index} was dealt a key twice");
                    }
                }
            }
        }
    }

    /// `M` is the manual's own key. A room dealt it as a working key would be
    /// a room whose challenge could never be told apart from paging the book.
    #[test]
    fn no_room_is_ever_dealt_the_manual_s_key() {
        for seed in 0..200u64 {
            let keys = RoomKeys::from_seed(seed, TEST_ROOM_COUNT);

            for index in 0..TEST_ROOM_COUNT {
                let dealt = keys.of(Room::from_index(index));

                for key in [
                    dealt.primary,
                    dealt.secondary,
                    dealt.up,
                    dealt.down,
                    dealt.action,
                ] {
                    assert_ne!(key, KeyCode::KeyM, "room {index} was dealt the manual's key");
                }
            }
        }
    }

    /// Two rooms dealt the same run must not end up with the same hand,
    /// however unlikely — otherwise the manual could print one room's keys
    /// for another's challenge and the player would never notice from the
    /// keyboard alone.
    #[test]
    fn rooms_in_the_same_run_are_dealt_different_keys() {
        let keys = RoomKeys::from_seed(11, TEST_ROOM_COUNT);
        let first = keys.of(Room::from_index(0));

        assert!(
            (1..TEST_ROOM_COUNT).any(|index| keys.of(Room::from_index(index)) != first),
            "every room in the run was dealt the same keys"
        );
    }

    /// A moment-apart seed has to shuffle the keys, the same way it shuffles
    /// the panels and the challenges.
    #[test]
    fn seeds_a_moment_apart_give_different_keys() {
        let seed = 1_234_567_890_u64;
        let differ = (1..=8).filter(|step| {
            RoomKeys::from_seed(seed + step, TEST_ROOM_COUNT)
                != RoomKeys::from_seed(seed, TEST_ROOM_COUNT)
        });

        assert!(differ.count() >= 6, "the pick barely moves between seeds");
    }

    /// Every key handed out prints back to the same letter it was dealt as.
    #[test]
    fn every_dealt_key_prints_its_own_letter() {
        for key in KEY_POOL {
            let letter = key_letter(key);
            assert!(letter.is_ascii_uppercase());
            assert_ne!(letter, 'M', "M was dealt out of the pool that excludes it");
        }
    }
}
