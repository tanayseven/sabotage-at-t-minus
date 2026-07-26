use bevy::audio::{AudioSinkPlayback, Volume};
use bevy::prelude::*;

use crate::settings::Settings;

const TRACK_PATH: &str = "music.ogg";

/// The looping background track. Lives for exactly one run: spawned on entering
/// [`GameState::Playing`](crate::state::GameState::Playing) and despawned on
/// leaving it, so it keeps playing through the confirm-quit dialog (a substate
/// of `Playing`) and stops only when the player is back at the menu.
#[derive(Component)]
pub struct BackgroundMusic;

pub fn start_music(mut commands: Commands, assets: Res<AssetServer>, settings: Res<Settings>) {
    commands.spawn((
        BackgroundMusic,
        AudioPlayer::new(assets.load(TRACK_PATH)),
        PlaybackSettings::LOOP.with_volume(Volume::Linear(settings.music_volume)),
    ));
}

/// Applies an options-screen change to a track that is already playing, so the
/// slider is not limited to taking effect on the next run.
pub fn apply_music_volume(
    settings: Res<Settings>,
    mut sinks: Query<&mut AudioSink, With<BackgroundMusic>>,
) {
    if !settings.is_changed() {
        return;
    }

    for mut sink in &mut sinks {
        sink.set_volume(Volume::Linear(settings.music_volume));
    }
}

pub fn stop_music(mut commands: Commands, music: Query<Entity, With<BackgroundMusic>>) {
    for entity in &music {
        commands.entity(entity).despawn();
    }
}
