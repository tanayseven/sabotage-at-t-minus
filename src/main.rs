mod camera;
mod config;
mod countdown;
mod credits;
mod difficulty;
mod difficulty_screen;
mod door;
mod font;
mod gameover;
mod ladder;
mod launchpad;
mod level;
mod manual;
mod menu;
mod minigame_keys;
mod minigames;
mod mission_complete;
mod music;
mod options;
mod panel;
mod physics;
mod platform;
mod player;
mod player_animation;
mod portal;
mod props;
mod puzzles;
mod quit;
mod settings;
mod setup;
mod sign;
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
use crate::countdown::{MissionTimer, tick_countdown};
use crate::credits::{despawn_credits, open_credit_link_action, spawn_credits};
use crate::difficulty_screen::{
    despawn_difficulty_screen, difficulty_screen_action, difficulty_step_action,
    spawn_difficulty_screen, sync_difficulty_readout,
};
use crate::door::{leave_through_airlock, sync_airlock_lock_state, use_doors};
use crate::font::FontPlugin;
use crate::gameover::{despawn_game_over, game_over_action, spawn_game_over};
use crate::ladder::climb_ladder;
use crate::launchpad::{board_rocket, despawn_launchpad, leave_launchpad, spawn_launchpad};
use crate::level::{Level, react_to_minigame_result, reset_level};
use crate::manual::{
    ManualPage, despawn_manual, manual_controls, reset_manual_page, sync_manual_page,
};
use crate::menu::{despawn_menu, menu_action, spawn_menu};
use crate::minigame_keys::RoomKeys;
use crate::minigames::{
    clear_active_minigame, despawn_minigame_window, run_active_minigame, spawn_minigame_window,
};
use crate::mission_complete::{
    despawn_mission_complete, mission_complete_action, spawn_mission_complete,
};
use crate::music::{apply_music_volume, start_music, stop_music};
use crate::options::{
    back_to_menu, despawn_options, spawn_options, sync_volume_widgets, volume_step_action,
};
use crate::panel::{Panels, flip_switches, light_panel, sync_panel_status};
use crate::physics::{configure_physics, pause_physics, resume_physics};
use crate::player::{jump, move_player, probe_ground};
use crate::player_animation::animate_player;
use crate::portal::{emit_portal_sparks, enter_portal, pulse_portal, update_portal_sparks};
use crate::puzzles::RocketPuzzles;
use crate::quit::{despawn_quit_dialog, open_quit_dialog, quit_dialog_action, spawn_quit_dialog};
use crate::settings::Settings;
use crate::setup::{despawn_game, setup};
use crate::splash::{animate_splash, despawn_splash, skip_splash, spawn_splash};
use crate::state::{GameState, PlayingState};
use crate::ui::{GameFont, button_visuals, sync_ui_scale};

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
        FontPlugin,
    ));

    #[cfg(feature = "dev")]
    app.add_plugins(RapierDebugRenderPlugin::default());

    // Jersey 10 is the game's one typeface, baked in by `FontPlugin` above as
    // the default font slot — so the handle this resource wraps is that same
    // slot rather than a second, separately loaded copy.
    app.insert_resource(GameFont(Handle::default()));

    app.init_state::<GameState>()
        .add_sub_state::<PlayingState>()
        .init_resource::<Settings>()
        .init_resource::<MissionTimer>()
        .init_resource::<Level>()
        .init_resource::<Panels>()
        .init_resource::<RocketPuzzles>()
        .init_resource::<RoomKeys>()
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
        // The track plays here too, muted nowhere else the menu leads, so a
        // drag of the slider is heard against something rather than against
        // silence — otherwise `apply_music_volume` has nothing to prove it
        // works until the next run.
        .add_systems(OnEnter(GameState::Options), (spawn_options, start_music))
        .add_systems(
            Update,
            (volume_step_action, sync_volume_widgets, back_to_menu)
                .chain()
                .run_if(in_state(GameState::Options)),
        )
        .add_systems(OnExit(GameState::Options), (despawn_options, stop_music))
        .add_systems(OnEnter(GameState::Credits), spawn_credits)
        .add_systems(
            Update,
            (back_to_menu, open_credit_link_action).run_if(in_state(GameState::Credits)),
        )
        .add_systems(OnExit(GameState::Credits), despawn_credits)
        .add_systems(OnEnter(GameState::Difficulty), spawn_difficulty_screen)
        .add_systems(
            Update,
            (
                difficulty_step_action,
                sync_difficulty_readout,
                difficulty_screen_action,
            )
                .chain()
                .run_if(in_state(GameState::Difficulty)),
        )
        .add_systems(OnExit(GameState::Difficulty), despawn_difficulty_screen)
        .add_systems(OnEnter(GameState::Launchpad), spawn_launchpad)
        .add_systems(
            Update,
            (board_rocket, leave_launchpad).run_if(in_state(GameState::Launchpad)),
        )
        .add_systems(OnExit(GameState::Launchpad), despawn_launchpad)
        .add_systems(
            OnEnter(GameState::Playing),
            (
                // `setup` deals the run's puzzles and the panel off one seed,
                // so nothing else may insert either ahead of it.
                (reset_level, setup).chain(),
                start_music,
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
        .add_systems(
            Update,
            sync_airlock_lock_state.run_if(in_state(PlayingState::Running)),
        )
        // `E` throws a switch as well as working a door, and the panel is
        // mounted where the two are never both in reach, so these can share the
        // press without racing. Chained after the toggle so the lamps and the
        // HUD show this frame's throw rather than last frame's.
        .add_systems(
            Update,
            (flip_switches, light_panel, sync_panel_status)
                .chain()
                .run_if(in_state(PlayingState::Running)),
        )
        // Chained so page turns and closes are reflected the same frame they
        // are requested, both while running and while a minigame overlay is up.
        .add_systems(
            Update,
            (manual_controls, sync_manual_page)
                .chain()
                .run_if(in_state(PlayingState::Running).or_else(in_state(PlayingState::Minigame))),
        )
        // Runs after manual controls so `Escape` consumed by the manual does
        // not also trigger a quit dialog while the run is active.
        .add_systems(
            Update,
            open_quit_dialog
                .after(sync_manual_page)
                .run_if(in_state(PlayingState::Running).or_else(in_state(PlayingState::Minigame))),
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
            tick_countdown
                .run_if(in_state(PlayingState::Running).or_else(in_state(PlayingState::Minigame))),
        )
        .add_systems(
            Update,
            (pulse_portal, emit_portal_sparks, update_portal_sparks)
                .run_if(in_state(PlayingState::Running)),
        )
        .add_systems(Update, enter_portal.run_if(in_state(PlayingState::Running)))
        .add_systems(
            Update,
            react_to_minigame_result.run_if(in_state(PlayingState::Running)),
        )
        .add_systems(
            OnExit(GameState::Playing),
            (
                despawn_game,
                stop_music,
                reset_camera,
                clear_active_minigame,
            ),
        )
        .add_systems(
            OnEnter(PlayingState::Minigame),
            (spawn_minigame_window, pause_physics),
        )
        .add_systems(
            Update,
            run_active_minigame.run_if(in_state(PlayingState::Minigame)),
        )
        .add_systems(
            OnExit(PlayingState::Minigame),
            (despawn_minigame_window, resume_physics),
        )
        .add_systems(
            OnEnter(PlayingState::ConfirmQuit),
            (despawn_manual, spawn_quit_dialog, pause_physics),
        )
        .add_systems(
            Update,
            quit_dialog_action.run_if(in_state(PlayingState::ConfirmQuit)),
        )
        .add_systems(
            OnExit(PlayingState::ConfirmQuit),
            (despawn_quit_dialog, resume_physics),
        )
        .add_systems(
            OnEnter(PlayingState::GameOver),
            (despawn_manual, spawn_game_over, pause_physics),
        )
        .add_systems(
            Update,
            game_over_action.run_if(in_state(PlayingState::GameOver)),
        )
        .add_systems(
            OnExit(PlayingState::GameOver),
            (despawn_game_over, resume_physics),
        )
        .add_systems(
            OnEnter(PlayingState::MissionComplete),
            (despawn_manual, spawn_mission_complete, pause_physics),
        )
        .add_systems(OnExit(GameState::Playing), despawn_manual)
        .add_systems(
            Update,
            mission_complete_action.run_if(in_state(PlayingState::MissionComplete)),
        )
        .add_systems(
            OnExit(PlayingState::MissionComplete),
            (despawn_mission_complete, resume_physics),
        )
        .add_systems(Update, (button_visuals, sync_ui_scale, apply_music_volume))
        .run();
}
