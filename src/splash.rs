use bevy::prelude::*;

use crate::state::GameState;

const FADE_IN: f32 = 4.0;
const HOLD: f32 = 0.2;
const FADE_OUT: f32 = 1.5;
const SPLASH_DURATION: f32 = FADE_IN + HOLD + FADE_OUT;

const LOGO_PATH: &str = "gmtk-game-jam-splash.png";
const LOGO_ASPECT: f32 = 2470.0 / 1718.0;

const TRANSPARENT_WHITE: Color = Color::srgba(1.0, 1.0, 1.0, 0.0);

#[derive(Component)]
pub struct SplashScreen;

#[derive(Component)]
pub struct SplashFade;

#[derive(Resource)]
pub struct SplashTimer(Timer);

fn fade_alpha(elapsed: f32) -> f32 {
    if elapsed < FADE_IN {
        elapsed / FADE_IN
    } else if elapsed < FADE_IN + HOLD {
        1.0
    } else {
        (1.0 - (elapsed - FADE_IN - HOLD) / FADE_OUT).max(0.0)
    }
}

pub fn spawn_splash(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(SplashTimer(Timer::from_seconds(
        SPLASH_DURATION,
        TimerMode::Once,
    )));

    commands
        .spawn((
            SplashScreen,
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: px(24),
                ..default()
            },
            BackgroundColor(Color::BLACK),
        ))
        .with_children(|parent| {
            parent.spawn((
                SplashFade,
                Text::new("Created for"),
                TextFont {
                    font_size: FontSize::Px(48.0),
                    ..default()
                },
                TextColor(TRANSPARENT_WHITE),
            ));
            parent.spawn((
                SplashFade,
                ImageNode {
                    image: asset_server.load(LOGO_PATH),
                    color: TRANSPARENT_WHITE,
                    ..default()
                },
                Node {
                    width: percent(55),
                    aspect_ratio: Some(LOGO_ASPECT),
                    ..default()
                },
            ));
        });
}

pub fn animate_splash(
    time: Res<Time>,
    mut timer: ResMut<SplashTimer>,
    mut next_state: ResMut<NextState<GameState>>,
    mut texts: Query<&mut TextColor, With<SplashFade>>,
    mut images: Query<&mut ImageNode, With<SplashFade>>,
) {
    timer.0.tick(time.delta());

    let alpha = fade_alpha(timer.0.elapsed_secs());

    for mut color in &mut texts {
        color.0.set_alpha(alpha);
    }
    for mut image in &mut images {
        image.color.set_alpha(alpha);
    }

    if timer.0.is_finished() {
        next_state.set(GameState::Menu);
    }
}

pub fn skip_splash(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let pressed_anything =
        keys.get_just_pressed().next().is_some() || mouse.get_just_pressed().next().is_some();

    if pressed_anything {
        next_state.set(GameState::Menu);
    }
}

pub fn despawn_splash(mut commands: Commands, splash: Query<Entity, With<SplashScreen>>) {
    for entity in &splash {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<SplashTimer>();
}
