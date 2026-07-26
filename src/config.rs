pub const DESIGN_WIDTH: f32 = 1280.0;
pub const DESIGN_HEIGHT: f32 = 720.0;

pub const VIEW_WIDTH: f32 = DESIGN_WIDTH + WALL_THICKNESS * 2.0;
pub const VIEW_HEIGHT: f32 = DESIGN_HEIGHT + WALL_THICKNESS * 2.0;

pub const PIXELS_PER_METER: f32 = 100.0;
pub const GRAVITY: f32 = 1800.0;

pub const PLAYER_SPEED: f32 = 400.0;
pub const JUMP_SPEED: f32 = 900.0;
pub const GROUND_PROBE: f32 = 4.0;

/// How fast the player goes up and down a ladder. Deliberately short of
/// [`PLAYER_SPEED`]: a climb is the slow part of crossing the rocket, and the
/// clock is what makes that a decision rather than a formality.
pub const CLIMB_SPEED: f32 = 260.0;

/// How tall the character stands on screen. The collider is the same height,
/// so what you see is what you collide with.
pub const PLAYER_HEIGHT: f32 = 64.0;

// Every character frame is a 128×128 image with the figure drawn roughly
// 81×97 px inside it, centred horizontally and standing on the frame's bottom
// edge. The whole frame has to be drawn to keep the figure's proportions, so
// the numbers below scale and shift it until the *figure* — not the frame —
// lines up with the collider.
const ART_FRAME: f32 = 128.0;
const ART_FIGURE_WIDTH: f32 = 81.0;
const ART_FIGURE_HEIGHT: f32 = 97.0;

/// Width of the collider, matching the figure's aspect ratio.
pub const PLAYER_WIDTH: f32 = PLAYER_HEIGHT * ART_FIGURE_WIDTH / ART_FIGURE_HEIGHT;

/// On-screen size of a whole character frame, transparent margins included.
pub const PLAYER_FRAME_SIZE: f32 = ART_FRAME * PLAYER_HEIGHT / ART_FIGURE_HEIGHT;

/// Where the entity's origin sits inside that frame, as an [`Anchor`] — a
/// normalised offset from the frame's centre, `-0.5` being its bottom edge.
/// The figure's feet are on that bottom edge, and the collider's own bottom is
/// half its height below the origin, so lifting the anchor by exactly that much
/// stands the figure on the floor of its collider instead of sinking it.
///
/// [`Anchor`]: bevy::sprite::Anchor
pub const PLAYER_ART_ANCHOR: f32 = -0.5 + (PLAYER_HEIGHT / 2.0) / PLAYER_FRAME_SIZE;

/// How long a run lasts before the launch clock hits zero.
pub const MISSION_SECONDS: f32 = 150.0;
/// Remaining time at which the HUD countdown switches to the warning colour.
pub const WARNING_SECONDS: f32 = 30.0;
/// Remaining time at which the HUD countdown switches to the alert colour and
/// starts beating like a heart.
pub const URGENT_SECONDS: f32 = 10.0;

pub const WALL_THICKNESS: f32 = 40.0;
pub const PLATFORM_HEIGHT: f32 = 32.0;

/// How far the camera zooms in inside the rocket. Gentle enough to keep the
/// deck above and the one below in frame while the player crosses a room.
pub const INTERIOR_ZOOM: f32 = 1.4;
/// Exponential decay rate for the tracking camera, per second. High enough to
/// stay glued to a sprint, low enough to smooth out landings.
pub const CAMERA_DECAY: f32 = 14.0;
