use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::ladder::Climbing;
use crate::player::{Grounded, Player};

const IDLE_FRAME: &str = "player/character_green_idle.png";
const JUMP_FRAME: &str = "player/character_green_jump.png";
const WALK_FRAMES: [&str; 2] = [
    "player/character_green_walk_a.png",
    "player/character_green_walk_b.png",
];
const CLIMB_FRAMES: [&str; 2] = [
    "player/character_green_climb_a.png",
    "player/character_green_climb_b.png",
];

/// How long each of the two walk frames is held. Two frames at this rate read
/// as a stride at `PLAYER_SPEED`; much slower and the character moonwalks.
const WALK_FRAME_SECONDS: f32 = 0.12;
/// The climb is slower than the walk, so its frames are held longer — at the
/// walk's rate a hand over hand up a ladder reads as a scramble.
const CLIMB_FRAME_SECONDS: f32 = 0.18;

/// Below this speed the character is treated as standing still, so that being
/// nudged by a crate doesn't set it walking on the spot. Read against the
/// vertical axis too, where it does the same for a player hanging on a ladder.
const WALKING_SPEED: f32 = 5.0;

/// Which drawing the character is showing. The jump frame covers falling too —
/// there is no separate fall pose in the art.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Pose {
    Idle,
    Walking,
    Airborne,
    Climbing,
}

impl Pose {
    /// Climbing wins over everything: a player on a ladder is off the ground by
    /// the ray probe's reckoning, and drawing them mid-jump would be a lie.
    fn of(climbing: bool, grounded: bool, horizontal_speed: f32) -> Self {
        if climbing {
            Pose::Climbing
        } else if !grounded {
            Pose::Airborne
        } else if horizontal_speed.abs() > WALKING_SPEED {
            Pose::Walking
        } else {
            Pose::Idle
        }
    }

    /// How long one frame of this pose's cycle is held. The still poses have no
    /// cycle, so what they answer never gets used.
    fn frame_seconds(self) -> f32 {
        match self {
            Pose::Climbing => CLIMB_FRAME_SECONDS,
            _ => WALK_FRAME_SECONDS,
        }
    }
}

/// The character's frames, plus enough state to know which one to show. Lives
/// on the player entity alongside its [`Sprite`].
#[derive(Component)]
pub struct PlayerAnimation {
    idle: Handle<Image>,
    jump: Handle<Image>,
    walk: [Handle<Image>; 2],
    climb: [Handle<Image>; 2],
    pose: Pose,
    step: usize,
    timer: Timer,
}

impl PlayerAnimation {
    pub fn load(assets: &AssetServer) -> Self {
        Self {
            idle: assets.load(IDLE_FRAME),
            jump: assets.load(JUMP_FRAME),
            walk: WALK_FRAMES.map(|path| assets.load(path)),
            climb: CLIMB_FRAMES.map(|path| assets.load(path)),
            pose: Pose::Idle,
            step: 0,
            timer: Timer::from_seconds(WALK_FRAME_SECONDS, TimerMode::Repeating),
        }
    }

    /// The two frames the current pose alternates between, if it is one that
    /// moves at all.
    fn cycle(&self) -> Option<&[Handle<Image>; 2]> {
        match self.pose {
            Pose::Walking => Some(&self.walk),
            Pose::Climbing => Some(&self.climb),
            Pose::Idle | Pose::Airborne => None,
        }
    }

    /// The frame to show right now.
    pub fn frame(&self) -> Handle<Image> {
        match self.cycle() {
            Some(frames) => frames[self.step].clone(),
            None if self.pose == Pose::Airborne => self.jump.clone(),
            None => self.idle.clone(),
        }
    }

    /// Switches pose, restarting the cycle whenever the character takes one up
    /// so a stride — or a climb — always begins on its first frame.
    fn set_pose(&mut self, pose: Pose) {
        if self.pose == pose {
            return;
        }

        self.pose = pose;
        self.step = 0;
        self.timer
            .set_duration(core::time::Duration::from_secs_f32(pose.frame_seconds()));
        self.timer.reset();
    }

    /// `moving` is what stops a character held still in a moving pose from
    /// working its frames anyway: a player resting on a ladder is climbing, but
    /// they are not going anywhere, and they should not be drawn as if they are.
    fn advance(&mut self, delta: core::time::Duration, moving: bool) {
        let Some(frames) = self.cycle() else {
            return;
        };
        if !moving {
            return;
        }

        let length = frames.len();
        self.timer.tick(delta);
        self.step = (self.step + self.timer.times_finished_this_tick() as usize) % length;
    }
}

/// Picks the pose from what the player is actually doing, ticks the cycle, and
/// turns the character to face the way it is moving.
pub fn animate_player(
    time: Res<Time>,
    mut players: Query<
        (
            &Velocity,
            &Grounded,
            &Climbing,
            &mut Sprite,
            &mut PlayerAnimation,
        ),
        With<Player>,
    >,
) {
    for (velocity, grounded, climbing, mut sprite, mut animation) in &mut players {
        let speed = velocity.linear.x;
        let pose = Pose::of(climbing.0, grounded.0, speed);

        // A climb is worked by how fast the player is going *up*, everything
        // else by how fast it is going along.
        let moving = match pose {
            Pose::Climbing => velocity.linear.y.abs() > WALKING_SPEED,
            _ => true,
        };

        animation.set_pose(pose);
        animation.advance(time.delta(), moving);

        // The art faces right, so only leftward movement needs mirroring. A
        // character that has stopped keeps facing whichever way it last went,
        // and one on a ladder faces the ladder rather than the room.
        if pose != Pose::Climbing && speed.abs() > WALKING_SPEED {
            sprite.flip_x = speed < 0.0;
        }

        sprite.image = animation.frame();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::PLAYER_SPEED;
    use core::time::Duration;

    /// Handles are only compared, never loaded, so the default ones are enough.
    fn animation() -> PlayerAnimation {
        PlayerAnimation {
            idle: Handle::default(),
            jump: Handle::default(),
            walk: [Handle::default(), Handle::default()],
            climb: [Handle::default(), Handle::default()],
            pose: Pose::Idle,
            step: 0,
            timer: Timer::from_seconds(WALK_FRAME_SECONDS, TimerMode::Repeating),
        }
    }

    fn walk_for(seconds: f32) -> usize {
        let mut animation = animation();
        animation.set_pose(Pose::Walking);
        animation.advance(Duration::from_secs_f32(seconds), true);
        animation.step
    }

    #[test]
    fn pose_follows_what_the_player_is_doing() {
        assert_eq!(Pose::of(false, true, 0.0), Pose::Idle);
        assert_eq!(Pose::of(false, true, PLAYER_SPEED), Pose::Walking);
        assert_eq!(Pose::of(false, true, -PLAYER_SPEED), Pose::Walking);
        // Airborne wins even when moving sideways: there is no walk-in-air pose.
        assert_eq!(Pose::of(false, false, PLAYER_SPEED), Pose::Airborne);
        assert_eq!(Pose::of(false, false, 0.0), Pose::Airborne);
    }

    /// A player on a ladder has nothing under their feet, so without climbing
    /// taking priority they would be drawn falling all the way up.
    #[test]
    fn climbing_beats_being_off_the_ground() {
        assert_eq!(Pose::of(true, false, 0.0), Pose::Climbing);
        assert_eq!(Pose::of(true, false, PLAYER_SPEED), Pose::Climbing);
        // And beats standing on the deck plate at the top of the ladder.
        assert_eq!(Pose::of(true, true, 0.0), Pose::Climbing);
    }

    #[test]
    fn a_gentle_nudge_does_not_count_as_walking() {
        assert_eq!(Pose::of(false, true, WALKING_SPEED / 2.0), Pose::Idle);
    }

    #[test]
    fn the_walk_cycle_alternates_and_wraps() {
        assert_eq!(walk_for(WALK_FRAME_SECONDS / 2.0), 0);
        assert_eq!(walk_for(WALK_FRAME_SECONDS * 1.5), 1);
        assert_eq!(walk_for(WALK_FRAME_SECONDS * 2.5), 0);
        // A long stall must not skip past the cycle into a panic.
        assert_eq!(walk_for(WALK_FRAME_SECONDS * 101.5), 1);
    }

    #[test]
    fn the_climb_cycle_runs_at_its_own_rate() {
        let mut animation = animation();
        animation.set_pose(Pose::Climbing);

        // Held longer than a walk frame, so what would have turned a stride over
        // leaves the climb on its first frame.
        animation.advance(Duration::from_secs_f32(WALK_FRAME_SECONDS * 1.2), true);
        assert_eq!(animation.step, 0);

        animation.advance(Duration::from_secs_f32(CLIMB_FRAME_SECONDS), true);
        assert_eq!(animation.step, 1);
    }

    /// Hanging on a ladder is still climbing, but it is not going anywhere.
    #[test]
    fn resting_on_a_ladder_does_not_work_the_climb() {
        let mut animation = animation();
        animation.set_pose(Pose::Climbing);
        animation.advance(Duration::from_secs_f32(CLIMB_FRAME_SECONDS * 3.0), false);

        assert_eq!(animation.step, 0);
    }

    #[test]
    fn only_the_moving_poses_advance_a_cycle() {
        for pose in [Pose::Idle, Pose::Airborne] {
            let mut animation = animation();
            animation.set_pose(pose);
            animation.advance(Duration::from_secs_f32(WALK_FRAME_SECONDS * 3.0), true);
            assert_eq!(animation.step, 0);
        }
    }

    #[test]
    fn every_stride_starts_on_its_first_frame() {
        let mut animation = animation();
        animation.set_pose(Pose::Walking);
        animation.advance(Duration::from_secs_f32(WALK_FRAME_SECONDS * 1.5), true);
        assert_eq!(animation.step, 1);

        // Stopping and setting off again restarts the cycle rather than
        // resuming halfway through it.
        animation.set_pose(Pose::Idle);
        animation.set_pose(Pose::Walking);
        assert_eq!(animation.step, 0);
    }
}
