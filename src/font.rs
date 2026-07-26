use bevy::asset::AssetId;
use bevy::prelude::*;
use bevy::text::Font;

/// Jersey 10 (SIL Open Font License 1.1) — the game's one typeface. It is a
/// pixel font, so it only looks right at whole-pixel sizes; the UI already
/// sets every size in `Px`.
///
/// Baked into the binary rather than loaded from `assets/` because the default
/// font has to exist before the very first frame lays out any text, and an
/// `AssetServer` load is asynchronous — on the web especially, a frame or two
/// of fallback glyphs would flash by first.
const JERSEY_10: &[u8] = include_bytes!("../assets/fonts/Jersey10-Regular.ttf");

/// Installs Jersey 10 as the font every `TextFont` gets by default.
///
/// `TextFont::default().font` resolves to `AssetId::default()`, which is the
/// slot Bevy's own `default_font` feature would fill with FiraMono. That
/// feature is off, so this is the only thing that ever writes the slot, and no
/// call site has to name a font handle.
pub struct FontPlugin;

impl Plugin for FontPlugin {
    fn build(&self, app: &mut App) {
        let mut fonts = app.world_mut().resource_mut::<Assets<Font>>();
        fonts
            .insert(AssetId::default(), Font::from_bytes(JERSEY_10.to_vec()))
            .expect("the default font slot is only ever written here");
    }
}
