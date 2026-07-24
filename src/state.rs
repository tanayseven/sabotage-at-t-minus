use bevy::prelude::*;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    Splash,
    Menu,
    Options,
    Credits,
    /// The pad the player boards the rocket from. Picking "Play" lands here,
    /// and walking into the rocket's hatch is what starts the run.
    Launchpad,
    Playing,
}

/// Exists only while we are in [`GameState::Playing`]. The confirm-quit dialog
/// is modal, so it gets its own state rather than a flag: gameplay systems and
/// the physics pipeline stand down for as long as the dialog is up.
#[derive(SubStates, Default, Debug, Clone, PartialEq, Eq, Hash)]
#[source(GameState = GameState::Playing)]
pub enum PlayingState {
    #[default]
    Running,
    ConfirmQuit,
}
