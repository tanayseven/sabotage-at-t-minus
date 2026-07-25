use bevy::camera::ScalingMode;
use bevy::prelude::*;

use crate::config::{CAMERA_DECAY, DESIGN_HEIGHT, DESIGN_WIDTH, VIEW_HEIGHT, VIEW_WIDTH};
use crate::level::{CameraMode, Level};
use crate::player::Player;

const BACKDROP_COLOR: Color = Color::srgb(0.10, 0.11, 0.14);
const BACKDROP_Z: f32 = -10.0;

/// The flat colour behind everything. It is stretched to cover wherever the
/// camera can reach, so a level the camera pans across never shows the void
/// past the edge of the backdrop.
#[derive(Component)]
pub struct Backdrop;

type Cameras<'w, 's> =
    Query<'w, 's, (&'static mut Projection, &'static mut Transform), With<Camera2d>>;
type Backdrops<'w, 's> = Query<
    'w,
    's,
    (&'static mut Sprite, &'static mut Transform),
    (With<Backdrop>, Without<Camera2d>),
>;

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, projection(CameraMode::Fixed)));

    commands.spawn((
        Backdrop,
        Sprite {
            color: BACKDROP_COLOR,
            custom_size: Some(Vec2::new(DESIGN_WIDTH, DESIGN_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, BACKDROP_Z),
    ));
}

fn projection(mode: CameraMode) -> Projection {
    let zoom = mode.zoom();

    Projection::Orthographic(OrthographicProjection {
        scaling_mode: ScalingMode::AutoMin {
            min_width: VIEW_WIDTH / zoom,
            min_height: VIEW_HEIGHT / zoom,
        },
        ..OrthographicProjection::default_2d()
    })
}

/// Frames the level that was just built. Runs on any change to [`Level`], which
/// covers both starting a run and stepping through an exit part way through one.
pub fn apply_level_camera(level: Res<Level>, cameras: Cameras, backdrops: Backdrops) {
    // Opens on the player's drop point rather than sliding over from wherever
    // the last level left the camera.
    frame(level.camera(), level.player_spawn(), cameras, backdrops);
}

/// Puts the fixed, whole-screen framing back for the menu and the launch pad,
/// which are all built to the size of the viewport.
pub fn reset_camera(cameras: Cameras, backdrops: Backdrops) {
    frame(CameraMode::Fixed, Vec2::ZERO, cameras, backdrops);
}

fn frame(mode: CameraMode, focus: Vec2, mut cameras: Cameras, mut backdrops: Backdrops) {
    let (backdrop_size, backdrop_centre) = match mode {
        CameraMode::Fixed => (Vec2::new(DESIGN_WIDTH, DESIGN_HEIGHT), Vec2::ZERO),
        CameraMode::Follow { bounds, .. } => (bounds.size(), bounds.center()),
    };

    for (mut sprite, mut transform) in &mut backdrops {
        sprite.custom_size = Some(backdrop_size);
        transform.translation.x = backdrop_centre.x;
        transform.translation.y = backdrop_centre.y;
    }

    for (mut camera_projection, mut transform) in &mut cameras {
        *camera_projection = projection(mode);

        let position = match mode {
            CameraMode::Fixed => Vec2::ZERO,
            CameraMode::Follow { bounds, .. } => {
                clamp_to(focus, half_view(&camera_projection), bounds)
            }
        };

        transform.translation.x = position.x;
        transform.translation.y = position.y;
    }
}

/// Pans along with the player on levels that ask for it. Runs after Rapier has
/// written the player's new position, so the camera never trails a frame behind
/// what it is following.
pub fn follow_player(
    time: Res<Time>,
    level: Res<Level>,
    players: Query<&Transform, (With<Player>, Without<Camera2d>)>,
    mut cameras: Query<(&mut Transform, &Projection), With<Camera2d>>,
) {
    let CameraMode::Follow { bounds, .. } = level.camera() else {
        return;
    };

    let Ok(player) = players.single() else {
        return;
    };

    // Framerate-independent easing: the gap to the player decays exponentially,
    // so the camera keeps up with a sprint but still settles after a landing
    // instead of snapping to every bounce.
    let catch_up = 1.0 - (-CAMERA_DECAY * time.delta_secs()).exp();

    for (mut transform, camera_projection) in &mut cameras {
        let target = clamp_to(
            player.translation.truncate(),
            half_view(camera_projection),
            bounds,
        );
        let eased = transform.translation.truncate().lerp(target, catch_up);

        transform.translation.x = eased.x;
        transform.translation.y = eased.y;
    }
}

/// What the camera can actually see. It depends on the window's aspect: the
/// projection scales to fit, so only one of the two design dimensions is the
/// one being honoured at any given size.
fn half_view(camera_projection: &Projection) -> Vec2 {
    match camera_projection {
        Projection::Orthographic(orthographic) => orthographic.area.size() / 2.0,
        _ => Vec2::new(VIEW_WIDTH, VIEW_HEIGHT) / 2.0,
    }
}

/// Keeps the viewport inside `bounds`, so the level never slides off its own
/// edge. An axis the viewport is already wider than cannot be satisfied at all,
/// so it centres on the bounds rather than snapping between the two edges.
fn clamp_to(focus: Vec2, half_view: Vec2, bounds: Rect) -> Vec2 {
    let slack = bounds.size() / 2.0 - half_view;
    let centre = bounds.center();

    Vec2::new(
        clamp_axis(focus.x, centre.x, slack.x),
        clamp_axis(focus.y, centre.y, slack.y),
    )
}

fn clamp_axis(focus: f32, centre: f32, slack: f32) -> f32 {
    if slack <= 0.0 {
        centre
    } else {
        focus.clamp(centre - slack, centre + slack)
    }
}

#[cfg(test)]
mod tests {
    use super::clamp_to;
    use bevy::prelude::*;

    const BOUNDS: Rect = Rect {
        min: Vec2::new(-1000.0, -500.0),
        max: Vec2::new(1000.0, 500.0),
    };
    const HALF_VIEW: Vec2 = Vec2::new(400.0, 200.0);

    #[test]
    fn follows_the_player_away_from_the_edges() {
        let focus = Vec2::new(120.0, -40.0);

        assert_eq!(clamp_to(focus, HALF_VIEW, BOUNDS), focus);
    }

    #[test]
    fn stops_at_the_edge_of_the_level() {
        assert_eq!(
            clamp_to(Vec2::new(980.0, 480.0), HALF_VIEW, BOUNDS),
            Vec2::new(600.0, 300.0)
        );
        assert_eq!(
            clamp_to(Vec2::new(-980.0, -480.0), HALF_VIEW, BOUNDS),
            Vec2::new(-600.0, -300.0)
        );
    }

    #[test]
    fn centres_on_an_axis_the_viewport_overflows() {
        let too_wide = Vec2::new(1200.0, 200.0);

        assert_eq!(
            clamp_to(Vec2::new(900.0, 0.0), too_wide, BOUNDS),
            Vec2::ZERO
        );
    }
}
