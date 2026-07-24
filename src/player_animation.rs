use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::player::{Grounded, Player};

const IDLE_FRAME: &str = "player/character_green_idle.png";
const JUMP_FRAME: &str = "player/character_green_jump.png";
const WALK_FRAMES: [&str; 2] = [
    "player/character_green_walk_a.png",
    "player/character_green_walk_b.png",
];

/// How long each of the two walk frames is held. Two frames at this rate read
/// as a stride at `PLAYER_SPEED`; much slower and the character moonwalks.
const WALK_FRAME_SECONDS: f32 = 0.12;

/// Below this horizontal speed the character is treated as standing still, so
/// that being nudged by a crate doesn't set it walking on the spot.
const WALKING_SPEED: f32 = 5.0;

/// Which drawing the character is showing. The jump frame covers falling too —
/// there is no separate fall pose in the art.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Pose {
    Idle,
    Walking,
    Airborne,
}

impl Pose {
    fn of(grounded: bool, horizontal_speed: f32) -> Self {
        if !grounded {
            Pose::Airborne
        } else if horizontal_speed.abs() > WALKING_SPEED {
            Pose::Walking
        } else {
            Pose::Idle
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
            pose: Pose::Idle,
            step: 0,
            timer: Timer::from_seconds(WALK_FRAME_SECONDS, TimerMode::Repeating),
        }
    }

    /// The frame to show right now.
    pub fn frame(&self) -> Handle<Image> {
        match self.pose {
            Pose::Idle => self.idle.clone(),
            Pose::Airborne => self.jump.clone(),
            Pose::Walking => self.walk[self.step].clone(),
        }
    }

    /// Switches pose, restarting the walk cycle whenever the character starts
    /// walking so a stride always begins on its first frame.
    fn set_pose(&mut self, pose: Pose) {
        if self.pose == pose {
            return;
        }

        self.pose = pose;
        self.step = 0;
        self.timer.reset();
    }

    fn advance(&mut self, delta: core::time::Duration) {
        if self.pose != Pose::Walking {
            return;
        }

        self.timer.tick(delta);
        self.step = (self.step + self.timer.times_finished_this_tick() as usize) % self.walk.len();
    }
}

/// Picks the pose from what the player is actually doing, ticks the walk cycle,
/// and turns the character to face the way it is moving.
pub fn animate_player(
    time: Res<Time>,
    mut players: Query<(&Velocity, &Grounded, &mut Sprite, &mut PlayerAnimation), With<Player>>,
) {
    for (velocity, grounded, mut sprite, mut animation) in &mut players {
        let speed = velocity.linear.x;

        animation.set_pose(Pose::of(grounded.0, speed));
        animation.advance(time.delta());

        // The art faces right, so only leftward movement needs mirroring. A
        // character that has stopped keeps facing whichever way it last went.
        if speed.abs() > WALKING_SPEED {
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
            pose: Pose::Idle,
            step: 0,
            timer: Timer::from_seconds(WALK_FRAME_SECONDS, TimerMode::Repeating),
        }
    }

    fn walk_for(seconds: f32) -> usize {
        let mut animation = animation();
        animation.set_pose(Pose::Walking);
        animation.advance(Duration::from_secs_f32(seconds));
        animation.step
    }

    #[test]
    fn pose_follows_what_the_player_is_doing() {
        assert_eq!(Pose::of(true, 0.0), Pose::Idle);
        assert_eq!(Pose::of(true, PLAYER_SPEED), Pose::Walking);
        assert_eq!(Pose::of(true, -PLAYER_SPEED), Pose::Walking);
        // Airborne wins even when moving sideways: there is no walk-in-air pose.
        assert_eq!(Pose::of(false, PLAYER_SPEED), Pose::Airborne);
        assert_eq!(Pose::of(false, 0.0), Pose::Airborne);
    }

    #[test]
    fn a_gentle_nudge_does_not_count_as_walking() {
        assert_eq!(Pose::of(true, WALKING_SPEED / 2.0), Pose::Idle);
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
    fn only_walking_advances_the_cycle() {
        for pose in [Pose::Idle, Pose::Airborne] {
            let mut animation = animation();
            animation.set_pose(pose);
            animation.advance(Duration::from_secs_f32(WALK_FRAME_SECONDS * 3.0));
            assert_eq!(animation.step, 0);
        }
    }

    #[test]
    fn every_stride_starts_on_its_first_frame() {
        let mut animation = animation();
        animation.set_pose(Pose::Walking);
        animation.advance(Duration::from_secs_f32(WALK_FRAME_SECONDS * 1.5));
        assert_eq!(animation.step, 1);

        // Stopping and setting off again restarts the cycle rather than
        // resuming halfway through it.
        animation.set_pose(Pose::Idle);
        animation.set_pose(Pose::Walking);
        assert_eq!(animation.step, 0);
    }
}
