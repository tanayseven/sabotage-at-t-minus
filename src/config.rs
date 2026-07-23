//! Tuning constants for the whole game, grouped by what they configure.
//!
//! Everything spatial is in world units authored against
//! [`DESIGN_WIDTH`] × [`DESIGN_HEIGHT`], never the window's pixel size.

// --- Design resolution ---------------------------------------------------

/// The resolution the game is authored at. Sprite sizes and positions are in
/// these units; the camera stretches them to fill whatever window it gets.
pub const DESIGN_WIDTH: f32 = 1280.0;
pub const DESIGN_HEIGHT: f32 = 720.0;

// --- Physics -------------------------------------------------------------

/// World units per physical metre. Rapier is tuned for objects around 1 metre,
/// so telling it that the 64-unit player is roughly two thirds of a metre keeps
/// the solver in the range where its defaults behave.
pub const PIXELS_PER_METER: f32 = 100.0;

/// Deliberately heavier than real gravity at this scale (which would be about
/// 981 units/s²). Platformers feel sluggish at 1g; this gives a fall that
/// arrives when the player expects it.
pub const GRAVITY: f32 = 1800.0;

// --- Player --------------------------------------------------------------

pub const PLAYER_SPEED: f32 = 400.0;
/// Reaches an apex of `JUMP_SPEED² / (2 · GRAVITY)` ≈ 225 units, comfortably
/// clearing the height between the platforms below.
pub const JUMP_SPEED: f32 = 900.0;
pub const PLAYER_SIZE: f32 = 64.0;
/// How far below the player's feet to look for standing-room when deciding
/// whether a jump is allowed. Needs to be forgiving enough to survive the
/// small gap the solver leaves between resting bodies.
pub const GROUND_PROBE: f32 = 4.0;

// --- Level geometry ------------------------------------------------------

pub const WALL_THICKNESS: f32 = 40.0;
pub const PLATFORM_HEIGHT: f32 = 32.0;
