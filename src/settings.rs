use bevy::prelude::*;

/// Player-facing audio levels, edited on the options screen. Kept as a resource
/// rather than baked into the audio components so a change can be applied to a
/// track that is already playing.
#[derive(Resource)]
pub struct Settings {
    pub music_volume: f32,
    pub sfx_volume: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            // The music is mastered louder than the game needs it, so it sits
            // under the action rather than over it by default.
            music_volume: 0.4,
            sfx_volume: 0.7,
        }
    }
}

/// How much one press of a `-`/`+` button moves a level.
pub const VOLUME_STEP: f32 = 0.1;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum VolumeChannel {
    Music,
    Sfx,
}

impl VolumeChannel {
    pub const ALL: [Self; 2] = [Self::Music, Self::Sfx];

    pub fn label(self) -> &'static str {
        match self {
            Self::Music => "Music",
            Self::Sfx => "Sound",
        }
    }

    pub fn get(self, settings: &Settings) -> f32 {
        match self {
            Self::Music => settings.music_volume,
            Self::Sfx => settings.sfx_volume,
        }
    }

    pub fn adjust(self, settings: &mut Settings, delta: f32) {
        let level = match self {
            Self::Music => &mut settings.music_volume,
            Self::Sfx => &mut settings.sfx_volume,
        };
        *level = (*level + delta).clamp(0.0, 1.0);
    }
}
