//! Sabotage at T-Minus — entry point.
//!
//! Right now this is a scaffold: a window, a camera, and a square you can
//! drive around, so that every part of the build/ship pipeline has something
//! real to carry. Replace `game` with the actual game.
//!
//! The window is resizable and everything in it scales with it. Game code
//! should be written against [`DESIGN_WIDTH`] × [`DESIGN_HEIGHT`] world units
//! and never against the window's pixel size — see [`sync_ui_scale`] for how
//! the UI layer is kept in step with the camera.
//!
//! Movement and collision go through Rapier. The view is from the side: bodies
//! fall under gravity and land on the platforms spawned by [`spawn_platforms`].

use bevy::camera::ScalingMode;
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowResolution};
use bevy_rapier2d::prelude::*;

/// The resolution the game is authored at. Sprite sizes and positions are in
/// these units; the camera stretches them to fill whatever window it gets.
const DESIGN_WIDTH: f32 = 1280.0;
const DESIGN_HEIGHT: f32 = 720.0;

/// World units per physical metre. Rapier is tuned for objects around 1 metre,
/// so telling it that the 64-unit player is roughly two thirds of a metre keeps
/// the solver in the range where its defaults behave.
const PIXELS_PER_METER: f32 = 100.0;

/// Deliberately heavier than real gravity at this scale (which would be about
/// 981 units/s²). Platformers feel sluggish at 1g; this gives a fall that
/// arrives when the player expects it.
const GRAVITY: f32 = 1800.0;

const PLAYER_SPEED: f32 = 400.0;
/// Reaches an apex of `JUMP_SPEED² / (2 · GRAVITY)` ≈ 225 units, comfortably
/// clearing the height between the platforms below.
const JUMP_SPEED: f32 = 900.0;
const PLAYER_SIZE: f32 = 64.0;

const WALL_THICKNESS: f32 = 40.0;
const PLATFORM_HEIGHT: f32 = 32.0;
/// How far below the player's feet to look for standing-room when deciding
/// whether a jump is allowed. Needs to be forgiving enough to survive the
/// small gap the solver leaves between resting bodies.
const GROUND_PROBE: f32 = 4.0;

fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Sabotage at T-Minus".into(),
                resolution: WindowResolution::new(DESIGN_WIDTH as u32, DESIGN_HEIGHT as u32),
                resizable: true,
                canvas: Some("#game-canvas".into()),
                fit_canvas_to_parent: true,
                // Let the browser keep handling F5, ctrl+L and friends.
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }),
        RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(PIXELS_PER_METER),
    ));

    // Collider outlines are a debugging aid, not something players should see.
    #[cfg(feature = "dev")]
    app.add_plugins(RapierDebugRenderPlugin::default());

    app.add_systems(Startup, (configure_physics, setup))
        .add_systems(Update, (move_player, jump, sync_ui_scale))
        .run();
}

#[derive(Component)]
struct Player;

/// Overrides Rapier's default gravity with the value this game is tuned for.
fn configure_physics(mut configs: Query<&mut RapierConfiguration>) {
    for mut config in &mut configs {
        config.gravity = Vec2::new(0.0, -GRAVITY);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        // `AutoMin` keeps the aspect ratio and guarantees the whole design area
        // stays on screen: as the window grows, the world grows with it instead
        // of simply revealing more of it. A window with a different aspect ratio
        // than the design one shows extra world on the roomier axis rather than
        // letterboxing, so keep anything important away from the very edges.
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: DESIGN_WIDTH,
                min_height: DESIGN_HEIGHT,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));

    // Backdrop, sized to the design area so it doubles as a visible marker of
    // what is guaranteed to be on screen at any window size.
    commands.spawn((
        Sprite {
            color: Color::srgb(0.10, 0.11, 0.14),
            custom_size: Some(Vec2::new(DESIGN_WIDTH, DESIGN_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -10.0),
    ));

    spawn_walls(&mut commands);
    spawn_platforms(&mut commands);

    // Spawned above the leftmost platform, so the first thing the game shows is
    // the player falling and landing on it.
    commands.spawn((
        Player,
        Sprite {
            color: Color::srgb(0.9, 0.35, 0.2),
            custom_size: Some(Vec2::splat(PLAYER_SIZE)),
            ..default()
        },
        Transform::from_xyz(-380.0, 260.0, 0.0),
        RigidBody::Dynamic,
        Collider::cuboid(PLAYER_SIZE / 2.0, PLAYER_SIZE / 2.0),
        // Driven by setting velocity directly, so the solver must not tip the
        // player over when it clips a corner.
        LockedAxes::ROTATION_LOCKED,
        Velocity::zero(),
        // Horizontal speed is assigned every frame, so surface friction would
        // only fight the controller.
        Friction::coefficient(0.0),
        // Stops the player tunnelling through a platform at the bottom of a
        // long fall, when per-frame movement can exceed the platform's depth.
        Ccd::enabled(),
    ));

    // Something to shove off the ledges, to make the physics visible at a glance.
    for (index, position) in [
        Vec2::new(-300.0, 40.0),
        Vec2::new(120.0, 300.0),
        Vec2::new(200.0, 300.0),
        Vec2::new(430.0, 120.0),
    ]
    .into_iter()
    .enumerate()
    {
        let size = 44.0 + (index as f32 * 8.0);
        commands.spawn((
            Sprite {
                color: Color::srgb(0.35, 0.45, 0.6),
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
            Transform::from_xyz(position.x, position.y, 0.0),
            RigidBody::Dynamic,
            Collider::cuboid(size / 2.0, size / 2.0),
            Restitution::coefficient(0.1),
        ));
    }

    commands.spawn((
        Text::new("Sabotage at T-Minus\nA/D or arrows to move, W / space to jump"),
        Node {
            position_type: PositionType::Absolute,
            top: px(16),
            left: px(16),
            ..default()
        },
    ));
}

/// Static colliders around the design area. The bottom one doubles as the
/// ground; the rest stop anything leaving the region the camera guarantees is
/// visible.
fn spawn_walls(commands: &mut Commands) {
    let half_width = DESIGN_WIDTH / 2.0;
    let half_height = DESIGN_HEIGHT / 2.0;
    let half_thickness = WALL_THICKNESS / 2.0;

    // Centre position and half-extents for each of the four walls, each sitting
    // just outside the design area so its inner face lines up with the edge.
    let walls = [
        (
            Vec2::new(0.0, half_height + half_thickness),
            Vec2::new(half_width + WALL_THICKNESS, half_thickness),
        ),
        (
            Vec2::new(0.0, -half_height - half_thickness),
            Vec2::new(half_width + WALL_THICKNESS, half_thickness),
        ),
        (
            Vec2::new(-half_width - half_thickness, 0.0),
            Vec2::new(half_thickness, half_height + WALL_THICKNESS),
        ),
        (
            Vec2::new(half_width + half_thickness, 0.0),
            Vec2::new(half_thickness, half_height + WALL_THICKNESS),
        ),
    ];

    for (centre, half_extents) in walls {
        commands.spawn((
            Sprite {
                color: Color::srgb(0.18, 0.20, 0.26),
                custom_size: Some(half_extents * 2.0),
                ..default()
            },
            Transform::from_xyz(centre.x, centre.y, -5.0),
            RigidBody::Fixed,
            Collider::cuboid(half_extents.x, half_extents.y),
        ));
    }
}

/// The ledges the player lands on and jumps between. Vertical gaps are kept
/// under the jump apex so every platform is reachable from the one before it.
fn spawn_platforms(commands: &mut Commands) {
    let platforms = [
        (Vec2::new(-380.0, -120.0), 360.0),
        (Vec2::new(40.0, 60.0), 300.0),
        (Vec2::new(430.0, -40.0), 260.0),
    ];

    for (centre, width) in platforms {
        commands.spawn((
            Sprite {
                color: Color::srgb(0.26, 0.30, 0.38),
                custom_size: Some(Vec2::new(width, PLATFORM_HEIGHT)),
                ..default()
            },
            Transform::from_xyz(centre.x, centre.y, -1.0),
            RigidBody::Fixed,
            Collider::cuboid(width / 2.0, PLATFORM_HEIGHT / 2.0),
        ));
    }
}

/// Scales the UI layer by the same factor the camera scales the world by.
///
/// Bevy's UI is laid out in window pixels and so ignores the camera projection
/// entirely; without this, text and HUD nodes would stay a fixed pixel size
/// while the sprites around them grew. Mirroring `ScalingMode::AutoMin`'s
/// factor here keeps the two layers locked together.
fn sync_ui_scale(
    windows: Query<&Window, (With<PrimaryWindow>, Changed<Window>)>,
    mut ui_scale: ResMut<UiScale>,
) {
    // Nothing to do on frames where the window hasn't moved or resized.
    let Ok(window) = windows.single() else {
        return;
    };

    let scale = (window.width() / DESIGN_WIDTH).min(window.height() / DESIGN_HEIGHT);
    if !scale.is_finite() || scale <= 0.0 {
        return;
    }

    // Writing through `ResMut` flags the UI for a full relayout, so only do it
    // when the factor has actually moved.
    if (ui_scale.0 - scale).abs() > f32::EPSILON {
        ui_scale.0 = scale;
    }
}

/// Steers the player horizontally, leaving the vertical component to gravity
/// and [`jump`]. Writing `Transform` directly would teleport the body through
/// anything in the way.
fn move_player(keys: Res<ButtonInput<KeyCode>>, mut players: Query<&mut Velocity, With<Player>>) {
    let mut direction = 0.0;
    if keys.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]) {
        direction -= 1.0;
    }
    if keys.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]) {
        direction += 1.0;
    }

    for mut velocity in &mut players {
        // Assigning rather than accumulating means no input is a full stop:
        // this is a character, not a crate, and it should answer the keyboard
        // directly. `linear.y` is left alone so gravity still applies.
        velocity.linear.x = direction * PLAYER_SPEED;
    }
}

/// Launches the player upwards, but only with something solid underfoot.
///
/// Groundedness is a short ray straight down from the player's centre. The ray
/// excludes the player's own collider, so `solid` casting from inside it is
/// safe.
fn jump(
    keys: Res<ButtonInput<KeyCode>>,
    rapier: ReadRapierContext,
    mut players: Query<(Entity, &Transform, &mut Velocity), With<Player>>,
) {
    if !keys.any_just_pressed([KeyCode::Space, KeyCode::KeyW, KeyCode::ArrowUp]) {
        return;
    }

    let Ok(context) = rapier.single() else {
        return;
    };

    for (entity, transform, mut velocity) in &mut players {
        let grounded = context
            .cast_ray(
                transform.translation.truncate(),
                Vec2::NEG_Y,
                PLAYER_SIZE / 2.0 + GROUND_PROBE,
                true,
                QueryFilter::default().exclude_collider(entity),
            )
            .is_some();

        if grounded {
            velocity.linear.y = JUMP_SPEED;
        }
    }
}
