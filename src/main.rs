mod camera;
mod config;
mod countdown;
mod credits;
mod door;
mod gameover;
mod ladder;
mod launchpad;
mod level;
mod manual;
mod menu;
mod music;
mod options;
mod physics;
mod platform;
mod player;
mod player_animation;
mod props;
mod quit;
mod settings;
mod setup;
mod splash;
mod state;
mod tiles;
mod ui;
mod wall;

use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_rapier2d::prelude::*;

use crate::camera::{apply_level_camera, follow_player, reset_camera, setup_camera};
use crate::config::{DESIGN_HEIGHT, DESIGN_WIDTH, PIXELS_PER_METER};
use crate::countdown::{MissionTimer, reset_mission_timer, tick_countdown};
use crate::credits::{despawn_credits, spawn_credits};
use crate::door::{leave_through_airlock, use_doors};
use crate::gameover::{despawn_game_over, game_over_action, spawn_game_over};
use crate::ladder::climb_ladder;
use crate::launchpad::{board_rocket, despawn_launchpad, leave_launchpad, spawn_launchpad};
use crate::level::{Level, reset_level};
use crate::manual::{
    ManualPage, despawn_manual, manual_controls, reset_manual_page, sync_manual_page,
};
use crate::menu::{despawn_menu, menu_action, spawn_menu};
use crate::music::{apply_music_volume, start_music, stop_music};
use crate::options::{
    back_to_menu, despawn_options, spawn_options, sync_volume_widgets, volume_step_action,
};
use crate::physics::{configure_physics, pause_physics, resume_physics};
use crate::player::{jump, move_player, probe_ground};
use crate::player_animation::animate_player;
use crate::quit::{despawn_quit_dialog, open_quit_dialog, quit_dialog_action, spawn_quit_dialog};
use crate::settings::Settings;
use crate::setup::{despawn_game, setup};
use crate::splash::{animate_splash, despawn_splash, skip_splash, spawn_splash};
use crate::state::{GameState, PlayingState};
use crate::ui::{button_visuals, sync_ui_scale};

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
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }),
        RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(PIXELS_PER_METER),
    ));

    #[cfg(feature = "dev")]
    app.add_plugins(RapierDebugRenderPlugin::default());

    app.init_state::<GameState>()
        .add_sub_state::<PlayingState>()
        .init_resource::<Settings>()
        .init_resource::<MissionTimer>()
        .init_resource::<Level>()
        .init_resource::<ManualPage>()
        .add_systems(Startup, (configure_physics, setup_camera))
        .add_systems(OnEnter(GameState::Splash), spawn_splash)
        .add_systems(
            Update,
            (animate_splash, skip_splash).run_if(in_state(GameState::Splash)),
        )
        .add_systems(OnExit(GameState::Splash), despawn_splash)
        .add_systems(OnEnter(GameState::Menu), spawn_menu)
        .add_systems(Update, menu_action.run_if(in_state(GameState::Menu)))
        .add_systems(OnExit(GameState::Menu), despawn_menu)
        .add_systems(OnEnter(GameState::Options), spawn_options)
        .add_systems(
            Update,
            (volume_step_action, sync_volume_widgets, back_to_menu)
                .chain()
                .run_if(in_state(GameState::Options)),
        )
        .add_systems(OnExit(GameState::Options), despawn_options)
        .add_systems(OnEnter(GameState::Credits), spawn_credits)
        .add_systems(Update, back_to_menu.run_if(in_state(GameState::Credits)))
        .add_systems(OnExit(GameState::Credits), despawn_credits)
        .add_systems(OnEnter(GameState::Launchpad), spawn_launchpad)
        .add_systems(
            Update,
            (board_rocket, leave_launchpad).run_if(in_state(GameState::Launchpad)),
        )
        .add_systems(OnExit(GameState::Launchpad), despawn_launchpad)
        .add_systems(
            OnEnter(GameState::Playing),
            (
                (reset_level, setup).chain(),
                start_music,
                reset_mission_timer,
                reset_manual_page,
            ),
        )
        // The character is driven the same way on the launch pad as in the
        // level, so this is registered once for both rather than twice. Chained
        // because each step reads what the one before it wrote: the ray probe
        // needs this frame's movement, the climb needs the probe, jumping needs
        // to know the climb has not already taken the W it is looking at, and
        // the animation needs all four to pick a pose. The launch pad has no
        // ladders on it, so the climb simply finds nothing to take hold of.
        .add_systems(
            Update,
            (
                move_player,
                probe_ground,
                climb_ladder,
                jump,
                animate_player,
            )
                .chain()
                .run_if(in_state(GameState::Launchpad).or_else(in_state(PlayingState::Running))),
        )
        // Working a door and stepping through it are two presses apart, but the
        // second can land on the frame the first opens the airlock, so they are
        // chained in that order rather than racing.
        .add_systems(
            Update,
            (use_doors, leave_through_airlock)
                .chain()
                .run_if(in_state(PlayingState::Running)),
        )
        // Chained, and in this order, for two reasons: a page turned this frame
        // should be on screen this frame rather than a frame behind the press,
        // and `Escape` closing the manual must be eaten before the quit dialog
        // reads it as an abort.
        .add_systems(
            Update,
            (manual_controls, sync_manual_page, open_quit_dialog)
                .chain()
                .run_if(in_state(PlayingState::Running)),
        )
        // Frames the level a run opens on.
        .add_systems(
            Update,
            apply_level_camera
                .run_if(in_state(GameState::Playing).and_then(resource_changed::<Level>)),
        )
        // In PostUpdate, so it reads the position Rapier has just written for
        // the player rather than last frame's.
        .add_systems(
            PostUpdate,
            follow_player
                .after(PhysicsSet::Writeback)
                .before(TransformSystems::Propagate)
                .run_if(in_state(PlayingState::Running)),
        )
        // Independent of the movement chain: the clock runs regardless of what
        // the player does this frame.
        .add_systems(
            Update,
            tick_countdown.run_if(in_state(PlayingState::Running)),
        )
        .add_systems(
            OnExit(GameState::Playing),
            (despawn_game, stop_music, reset_camera),
        )
        .add_systems(
            OnEnter(PlayingState::ConfirmQuit),
            (spawn_quit_dialog, pause_physics),
        )
        .add_systems(
            Update,
            quit_dialog_action.run_if(in_state(PlayingState::ConfirmQuit)),
        )
        .add_systems(
            OnExit(PlayingState::ConfirmQuit),
            (despawn_quit_dialog, resume_physics),
        )
        // The manual is an overlay rather than a state, so it is cleared on the
        // way out of a running run instead of on its own transition.
        .add_systems(OnExit(PlayingState::Running), despawn_manual)
        .add_systems(
            OnEnter(PlayingState::GameOver),
            (spawn_game_over, pause_physics),
        )
        .add_systems(
            Update,
            game_over_action.run_if(in_state(PlayingState::GameOver)),
        )
        .add_systems(
            OnExit(PlayingState::GameOver),
            (despawn_game_over, resume_physics),
        )
        .add_systems(Update, (button_visuals, sync_ui_scale, apply_music_volume))
        .run();
}
