use bevy::prelude::*;

use crate::state::PlayingState;

mod tap_challenge;

const OVERLAY_SCRIM: Color = Color::srgba(0.0, 0.0, 0.0, 0.35);
const WINDOW_FILL: Color = Color::srgb(1.0, 1.0, 1.0);
const WINDOW_BORDER: Color = Color::srgb(0.0, 0.0, 0.0);

const WINDOW_SIZE: f32 = 380.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinigameId {
    TapChallenge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinigameOutcome {
    Success,
    #[allow(dead_code)]
    Failure,
    TimedOut,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinigameConfig {
    pub id: MinigameId,
    pub time_limit_seconds: f32,
}

/// Common contract future minigames can implement.
pub trait MinigameInstance: Send + Sync + 'static {
    fn title(&self) -> &'static str;
    fn instructions(&self) -> &'static str;
    fn status(&self, remaining_seconds: f32) -> String;
    fn tick(&mut self, keys: &ButtonInput<KeyCode>) -> Option<MinigameOutcome>;
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct PendingMinigame(pub MinigameConfig);

#[derive(Resource)]
pub struct ActiveMinigame {
    pub id: MinigameId,
    pub game: Box<dyn MinigameInstance>,
    pub remaining_seconds: f32,
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct CompletedMinigame {
    pub id: MinigameId,
    pub outcome: MinigameOutcome,
}

#[derive(Component)]
pub struct MinigameWindow;

#[derive(Component)]
struct MinigameTitle;

#[derive(Component)]
struct MinigameInstructions;

#[derive(Component)]
pub struct MinigameStatus;

pub fn queue_minigame(commands: &mut Commands, config: MinigameConfig) {
    commands.insert_resource(PendingMinigame(config));
}

pub fn spawn_minigame_window(
    mut commands: Commands,
    pending: Option<Res<PendingMinigame>>,
    mut next_playing: ResMut<NextState<PlayingState>>,
) {
    let Some(pending) = pending else {
        // Nothing queued; return to running rather than trapping the player.
        next_playing.set(PlayingState::Running);
        return;
    };

    let config = pending.0;
    let id = config.id;
    let game = new_minigame(id);
    let title = game.title();
    let instructions = game.instructions();
    let status = game.status(config.time_limit_seconds);

    commands.insert_resource(ActiveMinigame {
        id,
        game,
        remaining_seconds: config.time_limit_seconds,
    });
    commands.remove_resource::<PendingMinigame>();

    commands
        .spawn((
            MinigameWindow,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(OVERLAY_SCRIM),
            GlobalZIndex(2),
        ))
        .with_children(|overlay| {
            overlay
                .spawn((
                    Node {
                        width: px(WINDOW_SIZE),
                        height: px(WINDOW_SIZE),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(px(22.0)),
                        border: UiRect::all(px(4.0)),
                        ..default()
                    },
                    BackgroundColor(WINDOW_FILL),
                    BorderColor::all(WINDOW_BORDER),
                ))
                .with_children(|window| {
                    window.spawn((
                        MinigameTitle,
                        Text::new(title),
                        TextFont {
                            font_size: FontSize::Px(32.0),
                            ..default()
                        },
                        TextColor(Color::BLACK),
                    ));
                    window.spawn((
                        MinigameInstructions,
                        Text::new(instructions),
                        TextFont {
                            font_size: FontSize::Px(20.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.2, 0.2, 0.2)),
                    ));
                    window.spawn((
                        MinigameStatus,
                        Text::new(status),
                        TextFont {
                            font_size: FontSize::Px(28.0),
                            ..default()
                        },
                        TextColor(Color::BLACK),
                    ));
                });
        });
}

pub fn run_active_minigame(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    active: Option<ResMut<ActiveMinigame>>,
    mut status_labels: Query<&mut Text, With<MinigameStatus>>,
    mut next_playing: ResMut<NextState<PlayingState>>,
) {
    let Some(mut active) = active else {
        next_playing.set(PlayingState::Running);
        return;
    };

    active.remaining_seconds -= time.delta_secs();

    if active.remaining_seconds <= 0.0 {
        commands.insert_resource(CompletedMinigame {
            id: active.id,
            outcome: MinigameOutcome::TimedOut,
        });
        next_playing.set(PlayingState::Running);
        return;
    }

    if let Some(outcome) = active.game.tick(&keys) {
        commands.insert_resource(CompletedMinigame {
            id: active.id,
            outcome,
        });
        next_playing.set(PlayingState::Running);
        return;
    }

    let status = active.game.status(active.remaining_seconds);
    for mut text in &mut status_labels {
        **text = status.clone();
    }
}

pub fn despawn_minigame_window(
    mut commands: Commands,
    windows: Query<Entity, With<MinigameWindow>>,
) {
    for entity in &windows {
        commands.entity(entity).despawn();
    }

    commands.remove_resource::<ActiveMinigame>();
}

fn new_minigame(id: MinigameId) -> Box<dyn MinigameInstance> {
    match id {
        MinigameId::TapChallenge => Box::new(tap_challenge::TapChallenge::new()),
    }
}
