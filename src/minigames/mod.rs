use bevy::prelude::*;
use bevy::audio::Volume;

use crate::settings::Settings;
use crate::state::PlayingState;
use crate::tiles::load_pixel_art;

mod broken_wire;
mod tap_challenge;

const OVERLAY_SCRIM: Color = Color::srgba(0.0, 0.0, 0.0, 0.35);
const WINDOW_FILL: Color = Color::srgb(1.0, 1.0, 1.0);
const WINDOW_BORDER: Color = Color::srgb(0.0, 0.0, 0.0);

const WINDOW_SIZE: f32 = 380.0;
const WIRES_BROKEN_PATH: &str = "wires-minigame/wires-broken.png";
const WIRES_JOINT_PATH: &str = "wires-minigame/wires-joint.png";
const WIRES_ZAP_PATH: &str = "wires-minigame/zap.ogg";
const WIRES_IMAGE_WIDTH: f32 = 512.0;
const WIRES_IMAGE_HEIGHT: f32 = 128.0;
const WIRES_LEFT_WIDTH: f32 = 255.0;
const WIRES_RIGHT_START: f32 = 285.0;
const WIRES_RIGHT_WIDTH: f32 = WIRES_IMAGE_WIDTH - WIRES_RIGHT_START;
const WIRES_CANVAS_WIDTH: f32 = 336.0;
const WIRES_CANVAS_HEIGHT: f32 = 170.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinigameId {
    TapChallenge,
    BrokenWire,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinigameOutcome {
    Success,
    #[allow(dead_code)]
    Failure,
    #[allow(dead_code)]
    TimedOut,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SequenceWireVisualState {
    pub separation: f32,
    pub jointed: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MinigameVisualState {
    Text(String),
    BrokenWires(SequenceWireVisualState),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinigameAudioCue {
    SequenceZap,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinigameConfig {
    pub id: MinigameId,
}

/// Common contract future minigames can implement.
pub trait MinigameInstance: Send + Sync + 'static {
    fn title(&self) -> &'static str;
    fn instructions(&self) -> &'static str;
    fn status(&self) -> String;
    fn visual_state(&self) -> MinigameVisualState {
        MinigameVisualState::Text(self.status())
    }
    fn take_audio_cues(&mut self) -> Vec<MinigameAudioCue> {
        Vec::new()
    }
    fn tick(&mut self, keys: &ButtonInput<KeyCode>, delta_seconds: f32) -> Option<MinigameOutcome>;
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct PendingMinigame(pub MinigameConfig);

#[derive(Resource)]
pub struct ActiveMinigame {
    pub id: MinigameId,
    pub game: Box<dyn MinigameInstance>,
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

#[derive(Component)]
pub(crate) struct SequenceWireSplitVisual;

#[derive(Component)]
pub(crate) struct SequenceWireLeft;

#[derive(Component)]
pub(crate) struct SequenceWireRight;

#[derive(Component)]
pub(crate) struct SequenceWireJoint;

pub fn queue_minigame(commands: &mut Commands, config: MinigameConfig) {
    commands.insert_resource(PendingMinigame(config));
}

pub fn spawn_minigame_window(
    mut commands: Commands,
    assets: Res<AssetServer>,
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
    let status = game.status();

    commands.insert_resource(ActiveMinigame {
        id,
        game,
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
                        overflow: Overflow::clip(),
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

                    if id == MinigameId::BrokenWire {
                        let broken = load_pixel_art(&assets, WIRES_BROKEN_PATH);
                        let joint = load_pixel_art(&assets, WIRES_JOINT_PATH);
                        let base_left = (WIRES_CANVAS_WIDTH - WIRES_IMAGE_WIDTH) * 0.5;
                        let base_right = base_left + WIRES_RIGHT_START;

                        window
                            .spawn(Node {
                                width: px(WIRES_CANVAS_WIDTH),
                                height: px(WIRES_CANVAS_HEIGHT),
                                position_type: PositionType::Relative,
                                ..default()
                            })
                            .with_children(|canvas| {
                                canvas
                                    .spawn((
                                        SequenceWireSplitVisual,
                                        Node {
                                            position_type: PositionType::Absolute,
                                            width: percent(100.0),
                                            height: px(WIRES_IMAGE_HEIGHT),
                                            top: px((WIRES_CANVAS_HEIGHT - WIRES_IMAGE_HEIGHT) * 0.5),
                                            ..default()
                                        },
                                    ))
                                    .with_children(|split| {
                                        split
                                            .spawn((
                                                SequenceWireLeft,
                                                Node {
                                                    position_type: PositionType::Absolute,
                                                    left: px(base_left),
                                                    width: px(WIRES_LEFT_WIDTH),
                                                    height: px(WIRES_IMAGE_HEIGHT),
                                                    overflow: Overflow::clip(),
                                                    ..default()
                                                },
                                            ))
                                            .with_children(|left| {
                                                left.spawn((
                                                    ImageNode {
                                                        image: broken.clone(),
                                                        ..default()
                                                    },
                                                    Node {
                                                        position_type: PositionType::Absolute,
                                                        width: px(WIRES_IMAGE_WIDTH),
                                                        height: px(WIRES_IMAGE_HEIGHT),
                                                        ..default()
                                                    },
                                                ));
                                            });

                                        split
                                            .spawn((
                                                SequenceWireRight,
                                                Node {
                                                    position_type: PositionType::Absolute,
                                                    left: px(base_right),
                                                    width: px(WIRES_RIGHT_WIDTH),
                                                    height: px(WIRES_IMAGE_HEIGHT),
                                                    overflow: Overflow::clip(),
                                                    ..default()
                                                },
                                            ))
                                            .with_children(|right| {
                                                right.spawn((
                                                    ImageNode {
                                                        image: broken,
                                                        ..default()
                                                    },
                                                    Node {
                                                        position_type: PositionType::Absolute,
                                                        left: px(-WIRES_RIGHT_START),
                                                        width: px(WIRES_IMAGE_WIDTH),
                                                        height: px(WIRES_IMAGE_HEIGHT),
                                                        ..default()
                                                    },
                                                ));
                                            });
                                    });

                                canvas.spawn((
                                    SequenceWireJoint,
                                    ImageNode {
                                        image: joint,
                                        ..default()
                                    },
                                    Node {
                                        position_type: PositionType::Absolute,
                                        left: px(base_left),
                                        top: px((WIRES_CANVAS_HEIGHT - WIRES_IMAGE_HEIGHT) * 0.5),
                                        width: px(WIRES_IMAGE_WIDTH),
                                        height: px(WIRES_IMAGE_HEIGHT),
                                        display: Display::None,
                                        ..default()
                                    },
                                ));
                            });
                    }
                });
        });
}

pub fn run_active_minigame(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    settings: Res<Settings>,
    assets: Res<AssetServer>,
    mut commands: Commands,
    active: Option<ResMut<ActiveMinigame>>,
    mut status_labels: Query<&mut Text, With<MinigameStatus>>,
    mut wire_nodes: ParamSet<(
        Query<&mut Node, With<SequenceWireLeft>>,
        Query<&mut Node, With<SequenceWireRight>>,
        Query<&mut Node, With<SequenceWireSplitVisual>>,
        Query<&mut Node, With<SequenceWireJoint>>,
    )>,
    mut next_playing: ResMut<NextState<PlayingState>>,
) {
    let Some(mut active) = active else {
        next_playing.set(PlayingState::Running);
        return;
    };

    if let Some(outcome) = active.game.tick(&keys, time.delta_secs()) {
        commands.insert_resource(CompletedMinigame {
            id: active.id,
            outcome,
        });
        next_playing.set(PlayingState::Running);
        return;
    }

    for cue in active.game.take_audio_cues() {
        match cue {
            MinigameAudioCue::SequenceZap => {
                commands.spawn((
                    AudioPlayer::new(assets.load(WIRES_ZAP_PATH)),
                    PlaybackSettings::DESPAWN.with_volume(Volume::Linear(settings.sfx_volume)),
                ));
            }
        }
    }

    match active.game.visual_state() {
        MinigameVisualState::Text(status) => {
            for mut text in &mut status_labels {
                **text = status.clone();
            }
        }
        MinigameVisualState::BrokenWires(visual) => {
            let status = active.game.status();
            for mut text in &mut status_labels {
                **text = status.clone();
            }

            let base_left = (WIRES_CANVAS_WIDTH - WIRES_IMAGE_WIDTH) * 0.5;
            let base_right = base_left + WIRES_RIGHT_START;
            let split_offset = visual.separation * 0.5;

            for mut left in &mut wire_nodes.p0() {
                left.left = px(base_left - split_offset);
            }

            for mut right in &mut wire_nodes.p1() {
                right.left = px(base_right + split_offset);
            }

            for mut split in &mut wire_nodes.p2() {
                split.display = if visual.jointed {
                    Display::None
                } else {
                    Display::Flex
                };
            }

            for mut joint in &mut wire_nodes.p3() {
                joint.display = if visual.jointed {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        }
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
        MinigameId::BrokenWire => Box::new(broken_wire::BrokenWire::new()),
    }
}
