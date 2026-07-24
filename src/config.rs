pub const DESIGN_WIDTH: f32 = 1280.0;
pub const DESIGN_HEIGHT: f32 = 720.0;

pub const VIEW_WIDTH: f32 = DESIGN_WIDTH + WALL_THICKNESS * 2.0;
pub const VIEW_HEIGHT: f32 = DESIGN_HEIGHT + WALL_THICKNESS * 2.0;

pub const PIXELS_PER_METER: f32 = 100.0;
pub const GRAVITY: f32 = 1800.0;

pub const PLAYER_SPEED: f32 = 400.0;
pub const JUMP_SPEED: f32 = 900.0;
pub const PLAYER_SIZE: f32 = 64.0;
pub const GROUND_PROBE: f32 = 4.0;

/// How long a run lasts before the launch clock hits zero.
pub const MISSION_SECONDS: f32 = 120.0;
/// Remaining time at which the HUD countdown switches to the alert colour.
pub const URGENT_SECONDS: f32 = 10.0;

pub const WALL_THICKNESS: f32 = 40.0;
pub const PLATFORM_HEIGHT: f32 = 32.0;
