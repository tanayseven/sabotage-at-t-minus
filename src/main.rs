//! Sabotage at T-Minus — entry point.
//!
//! Right now this is a scaffold: a window, a camera, and a square you can
//! drive around, so that every part of the build/ship pipeline has something
//! real to carry. Replace the placeholder scene with the actual game.
//!
//! The window is resizable and everything in it scales with it. Game code
//! should be written against [`config::DESIGN_WIDTH`] × [`config::DESIGN_HEIGHT`]
//! world units and never against the window's pixel size — see
//! [`ui::sync_ui_scale`] for how the UI layer is kept in step with the camera.
//!
//! Movement and collision go through Rapier. The view is from the side: bodies
//! fall under gravity and land on the platforms spawned by
//! [`platform::spawn_platforms`].

mod camera;
mod config;
mod physics;
mod platform;
mod player;
mod props;
mod setup;
mod ui;
mod wall;

use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_rapier2d::prelude::*;

use crate::config::{DESIGN_HEIGHT, DESIGN_WIDTH, PIXELS_PER_METER};
use crate::physics::configure_physics;
use crate::player::{jump, move_player};
use crate::setup::setup;
use crate::ui::sync_ui_scale;

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
