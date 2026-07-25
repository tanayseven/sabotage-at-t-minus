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

/// Exists only while we are in [`GameState::Playing`]. Everything that is modal
/// over a run gets a variant here rather than a flag: gameplay systems and the
/// physics pipeline stand down for as long as one of them is up. The repair
/// manual is deliberately not among them — it is an overlay the run carries on
/// underneath, so it has no state of its own.
#[derive(SubStates, Default, Debug, Clone, PartialEq, Eq, Hash)]
#[source(GameState = GameState::Playing)]
pub enum PlayingState {
    #[default]
    Running,
    /// A modal challenge opened by walking into one of the rocket's breaches.
    Minigame,
    ConfirmQuit,
    GameOver,
    /// Run finished successfully.
    MissionComplete,
}
