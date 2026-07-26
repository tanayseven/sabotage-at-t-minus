use bevy::audio::Volume;
use bevy::prelude::*;

use crate::level::Room;
use crate::minigame_keys::{MinigameKeys, RoomKeys};
use crate::settings::Settings;
use crate::state::PlayingState;
use crate::tiles::load_pixel_art;
use crate::ui::{ACCENT, GameFont};

mod broken_wire;
mod clean_engine;
mod coolant_valve;
mod pipe_flow;

pub(crate) use pipe_flow::{PIPE_COLS, PIPE_ROWS, PIPE_TILES, PipePiece};

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

const GAUGE_PATH: &str = "coolant-minigame/gauge.png";
const GAUGE_SEALED_PATH: &str = "coolant-minigame/gauge-sealed.png";
const GAUGE_NEEDLE_PATH: &str = "coolant-minigame/needle.png";
const GAUGE_HISS_PATH: &str = "coolant-minigame/hiss.ogg";
const GAUGE_SEALED_TING_PATH: &str = "coolant-minigame/success-ting.ogg";
const GAUGE_CANVAS_WIDTH: f32 = 336.0;
const GAUGE_CANVAS_HEIGHT: f32 = 170.0;
const GAUGE_WIDTH: f32 = GAUGE_CANVAS_WIDTH;
const GAUGE_HEIGHT: f32 = GAUGE_WIDTH * 0.25;
const GAUGE_NEEDLE_WIDTH: f32 = 16.0;

/// The track's inset and width in the gauge art: 432px inset 40px into a 512px
/// image. The needle is placed against these.
const GAUGE_TRACK_LEFT: f32 = 40.0 / 512.0;
const GAUGE_TRACK_WIDTH: f32 = 432.0 / 512.0;

const BORE_CLEAN_PATH: &str = "clean-engine/bore-clean.png";
const BORE_DIRTY_PATH: &str = "clean-engine/bore-dirty.png";

/// The bore's grid, which is *painted into both plates* — the art is drawn as
/// this many cells each way, so the challenge counts against these rather than
/// keeping a second set of its own that could drift out of step with the
/// pictures.
pub(crate) const BORE_ROWS: usize = 5;
pub(crate) const BORE_CELLS: u32 = 5;

/// A cell on screen. Both plates are drawn 16px to the cell, so this is a clean
/// 2× of the source — nearest-neighbour scaling by a whole number is what keeps
/// the pixels square instead of smearing some of them wider than others.
const BORE_CELL: f32 = 32.0;
const BORE_CANVAS: f32 = BORE_CELL * BORE_ROWS as f32;

/// How thick a line is drawn round the course the brush is standing on.
const BORE_BRUSH_BORDER: f32 = 3.0;

const PIPE_STRAIGHT_PATH: &str = "pipes-minigame/pipe-straight.png";
const PIPE_STRAIGHT_FLOW_PATH: &str = "pipes-minigame/pipe-straight-flow.png";
const PIPE_ELBOW_PATH: &str = "pipes-minigame/pipe-elbow.png";
const PIPE_ELBOW_FLOW_PATH: &str = "pipes-minigame/pipe-elbow-flow.png";
const PIPE_PORT_PATH: &str = "pipes-minigame/pipe-port.png";
const PIPE_PORT_FLOW_PATH: &str = "pipes-minigame/pipe-port-flow.png";

const PIPE_CANVAS_WIDTH: f32 = 336.0;
const PIPE_CANVAS_HEIGHT: f32 = 170.0;
/// One coupling on screen. The art is twice this, as the gauge and wires are.
const PIPE_TILE: f32 = 64.0;
const PIPE_PORT_WIDTH: f32 = PIPE_TILE / 2.0;
const PIPE_GRID_WIDTH: f32 = PIPE_TILE * PIPE_COLS as f32;
const PIPE_GRID_HEIGHT: f32 = PIPE_TILE * PIPE_ROWS as f32;
/// The grid with a stub bolted to either end, centred in the canvas.
const PIPE_GRID_LEFT: f32 =
    (PIPE_CANVAS_WIDTH - (PIPE_GRID_WIDTH + PIPE_PORT_WIDTH * 2.0)) / 2.0 + PIPE_PORT_WIDTH;
const PIPE_GRID_TOP: f32 = (PIPE_CANVAS_HEIGHT - PIPE_GRID_HEIGHT) / 2.0;
/// Behind the pipe rather than over it, so it never hides which way a piece
/// is lying.
const PIPE_WRENCH_WASH: Color = Color::srgba(0.1, 0.34, 0.74, 0.22);

/// The run has to fit its window or the end couplings are clipped away.
const _: () = assert!(PIPE_CANVAS_WIDTH >= PIPE_GRID_WIDTH + PIPE_PORT_WIDTH * 2.0);
const _: () = assert!(PIPE_CANVAS_HEIGHT >= PIPE_GRID_HEIGHT);

/// How many kinds of challenge there are. What anything handing out one
/// challenge per room counts against.
pub const MINIGAME_COUNT: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinigameId {
    BrokenWire,
    CoolantValve,
    CleanEngine,
    PipeFlow,
}

impl MinigameId {
    /// Every challenge in the game, in the order the manual documents them.
    /// A run installs one of each rather than picking from them, so this is the
    /// list the rooms are dealt out against.
    pub const ALL: [MinigameId; MINIGAME_COUNT] = [
        MinigameId::BrokenWire,
        MinigameId::CoolantValve,
        MinigameId::CleanEngine,
        MinigameId::PipeFlow,
    ];
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CoolantGaugeVisualState {
    /// Where the needle stands: 0 at the empty end of the track, 1 at the stop.
    pub fill: f32,
    pub sealed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EngineBoreVisualState {
    /// Cells cut clean in each course, counted from the left.
    pub cut: [u32; BORE_ROWS],
    /// The course the brush is standing on.
    pub row: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PipeTileVisual {
    pub piece: PipePiece,
    /// Quarter turns clockwise. The art is turned rather than shipped four
    /// times over.
    pub quarters: u8,
    pub flowing: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PipeRunVisualState {
    pub tiles: [PipeTileVisual; PIPE_TILES],
    /// Which coupling the wrench is on.
    pub cursor: usize,
    pub drained: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MinigameVisualState {
    Text(String),
    BrokenWires(SequenceWireVisualState),
    CoolantGauge(CoolantGaugeVisualState),
    EngineBore(EngineBoreVisualState),
    PipeRun(PipeRunVisualState),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinigameAudioCue {
    SequenceZap,
    CoolantVent,
    CoolantSealed,
    PipesMadeUp,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinigameConfig {
    pub id: MinigameId,
    /// Which room the breach opening this challenge is in — what its keys are
    /// drawn from. See [`crate::minigame_keys`].
    pub room: Room,
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

/// The moving parts of the gauge. One component with a variant rather than a
/// marker apiece, for the same reason `EngineBoreVisual` is one: each `&mut
/// Node` query costs a slot in the set below, and the set only has eight.
#[derive(Component)]
pub(crate) enum CoolantGaugeVisual {
    /// The plain face, shown while the line is still open.
    Face,
    /// The sealed face, swapped in for the plain one once it is shut off.
    SealedFace,
    /// The needle that rides along the track.
    Needle,
}

/// The moving parts of the bore. One component with a variant rather than a
/// marker apiece: every `&mut Node` query has to take a slot in the set below,
/// and the set has room for eight.
#[derive(Component)]
pub(crate) enum EngineBoreVisual {
    /// Clips the clean plate to how far along this course has been cut, so the
    /// dirty plate underneath shows through for the cells still fouled.
    Cut(usize),
    /// The line drawn round the course under the brush.
    Brush,
}

/// Which part of the pipe run an image node is. One marker for all of them, so
/// the run is redrawn from a single query rather than one per piece.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PipeSlot {
    Coupling(usize),
    Inlet,
    Drain,
}

/// The wash that marks the coupling the wrench is on.
#[derive(Component)]
pub(crate) struct PipeWrench;

fn pipe_tile_offset(index: usize) -> Vec2 {
    let row = index / PIPE_COLS;
    let column = index % PIPE_COLS;

    Vec2::new(
        PIPE_GRID_LEFT + column as f32 * PIPE_TILE,
        PIPE_GRID_TOP + row as f32 * PIPE_TILE,
    )
}

/// The needle's left edge for a place along the track, in canvas pixels. The
/// stem is centred on the reading rather than butted up against it.
fn needle_left(fill: f32) -> f32 {
    (GAUGE_TRACK_LEFT + fill.clamp(0.0, 1.0) * GAUGE_TRACK_WIDTH) * GAUGE_WIDTH
        - GAUGE_NEEDLE_WIDTH * 0.5
}

pub fn queue_minigame(commands: &mut Commands, config: MinigameConfig) {
    commands.insert_resource(PendingMinigame(config));
}

pub fn spawn_minigame_window(
    mut commands: Commands,
    assets: Res<AssetServer>,
    pending: Option<Res<PendingMinigame>>,
    active: Option<Res<ActiveMinigame>>,
    mut next_playing: ResMut<NextState<PlayingState>>,
    font: Res<GameFont>,
    room_keys: Res<RoomKeys>,
) {
    let (id, title, instructions, status) = if let Some(pending) = pending {
        let config = pending.0;
        let id = config.id;
        let game = new_minigame(id, room_keys.of(config.room));
        let title = game.title();
        let instructions = game.instructions();
        let status = game.status();

        commands.insert_resource(ActiveMinigame { id, game });
        commands.remove_resource::<PendingMinigame>();

        (id, title, instructions, status)
    } else if let Some(active) = active {
        (
            active.id,
            active.game.title(),
            active.game.instructions(),
            active.game.status(),
        )
    } else {
        // Nothing queued or active; return to running rather than trapping.
        next_playing.set(PlayingState::Running);
        return;
    };

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
            // Keep the HUD/manual above the scrim so timer and controls remain
            // readable and interactive while a minigame is open.
            GlobalZIndex(-1),
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
                        font.at(32.0),
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
                                            top: px(
                                                (WIRES_CANVAS_HEIGHT - WIRES_IMAGE_HEIGHT) * 0.5
                                            ),
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

                    if id == MinigameId::CleanEngine {
                        let dirty = load_pixel_art(&assets, BORE_DIRTY_PATH);
                        let clean = load_pixel_art(&assets, BORE_CLEAN_PATH);

                        window
                            .spawn(Node {
                                width: px(BORE_CANVAS),
                                height: px(BORE_CANVAS),
                                position_type: PositionType::Relative,
                                ..default()
                            })
                            .with_children(|canvas| {
                                // The fouled bore, laid down whole and never
                                // touched again. Everything above it is the
                                // clean plate being uncovered a cell at a time,
                                // so soot is what shows wherever nothing has
                                // been cut yet.
                                canvas.spawn((
                                    ImageNode {
                                        image: dirty,
                                        ..default()
                                    },
                                    Node {
                                        position_type: PositionType::Absolute,
                                        width: px(BORE_CANVAS),
                                        height: px(BORE_CANVAS),
                                        ..default()
                                    },
                                ));

                                // One clipping window per course, opened from
                                // the left as the course is cut. Each holds a
                                // full copy of the clean plate pushed up so the
                                // course's own band of it lands in the window —
                                // the same trick the wires use to show a slice
                                // of one picture.
                                for row in 0..BORE_ROWS {
                                    canvas
                                        .spawn((
                                            EngineBoreVisual::Cut(row),
                                            Node {
                                                position_type: PositionType::Absolute,
                                                left: px(0.0),
                                                top: px(row as f32 * BORE_CELL),
                                                width: px(0.0),
                                                height: px(BORE_CELL),
                                                overflow: Overflow::clip(),
                                                ..default()
                                            },
                                        ))
                                        .with_children(|cut| {
                                            cut.spawn((
                                                ImageNode {
                                                    image: clean.clone(),
                                                    ..default()
                                                },
                                                Node {
                                                    position_type: PositionType::Absolute,
                                                    left: px(0.0),
                                                    top: px(-(row as f32) * BORE_CELL),
                                                    width: px(BORE_CANVAS),
                                                    height: px(BORE_CANVAS),
                                                    ..default()
                                                },
                                            ));
                                        });
                                }

                                canvas.spawn((
                                    EngineBoreVisual::Brush,
                                    Node {
                                        position_type: PositionType::Absolute,
                                        left: px(0.0),
                                        top: px(0.0),
                                        width: px(BORE_CANVAS),
                                        height: px(BORE_CELL),
                                        border: UiRect::all(px(BORE_BRUSH_BORDER)),
                                        ..default()
                                    },
                                    BorderColor::all(ACCENT),
                                ));
                            });
                    }

                    if id == MinigameId::CoolantValve {
                        let face = load_pixel_art(&assets, GAUGE_PATH);
                        let sealed = load_pixel_art(&assets, GAUGE_SEALED_PATH);
                        let needle = load_pixel_art(&assets, GAUGE_NEEDLE_PATH);
                        let gauge_top = (GAUGE_CANVAS_HEIGHT - GAUGE_HEIGHT) * 0.5;

                        window
                            .spawn(Node {
                                width: px(GAUGE_CANVAS_WIDTH),
                                height: px(GAUGE_CANVAS_HEIGHT),
                                position_type: PositionType::Relative,
                                ..default()
                            })
                            .with_children(|canvas| {
                                // Stacked and swapped rather than re-imaged, so
                                // sealing never waits on an asset load.
                                for (marker_sealed, image, shown) in
                                    [(false, face, true), (true, sealed, false)]
                                {
                                    let node = Node {
                                        position_type: PositionType::Absolute,
                                        left: px(0.0),
                                        top: px(gauge_top),
                                        width: px(GAUGE_WIDTH),
                                        height: px(GAUGE_HEIGHT),
                                        display: if shown { Display::Flex } else { Display::None },
                                        ..default()
                                    };
                                    let visual = (ImageNode { image, ..default() }, node);

                                    if marker_sealed {
                                        canvas.spawn((CoolantGaugeVisual::SealedFace, visual));
                                    } else {
                                        canvas.spawn((CoolantGaugeVisual::Face, visual));
                                    }
                                }

                                canvas.spawn((
                                    CoolantGaugeVisual::Needle,
                                    ImageNode {
                                        image: needle,
                                        ..default()
                                    },
                                    Node {
                                        position_type: PositionType::Absolute,
                                        left: px(needle_left(0.0)),
                                        top: px(gauge_top),
                                        width: px(GAUGE_NEEDLE_WIDTH),
                                        height: px(GAUGE_HEIGHT),
                                        ..default()
                                    },
                                ));
                            });
                    }

                    if id == MinigameId::PipeFlow {
                        let straight = load_pixel_art(&assets, PIPE_STRAIGHT_PATH);
                        let port = load_pixel_art(&assets, PIPE_PORT_PATH);

                        window
                            .spawn(Node {
                                width: px(PIPE_CANVAS_WIDTH),
                                height: px(PIPE_CANVAS_HEIGHT),
                                position_type: PositionType::Relative,
                                ..default()
                            })
                            .with_children(|canvas| {
                                // First, so it lies under the pipe it marks.
                                canvas.spawn((
                                    PipeWrench,
                                    Node {
                                        position_type: PositionType::Absolute,
                                        left: px(pipe_tile_offset(0).x),
                                        top: px(pipe_tile_offset(0).y),
                                        width: px(PIPE_TILE),
                                        height: px(PIPE_TILE),
                                        border_radius: BorderRadius::all(px(6)),
                                        ..default()
                                    },
                                    BackgroundColor(PIPE_WRENCH_WASH),
                                ));

                                // One piece of art for both stubs: the drain is
                                // it turned about, so its transform is set here
                                // and the redraw never touches it.
                                for (slot, left, top, quarters) in [
                                    (
                                        PipeSlot::Inlet,
                                        PIPE_GRID_LEFT - PIPE_PORT_WIDTH,
                                        PIPE_GRID_TOP,
                                        0.0,
                                    ),
                                    (
                                        PipeSlot::Drain,
                                        PIPE_GRID_LEFT + PIPE_GRID_WIDTH,
                                        PIPE_GRID_TOP + PIPE_GRID_HEIGHT - PIPE_TILE,
                                        180.0,
                                    ),
                                ] {
                                    canvas.spawn((
                                        slot,
                                        ImageNode {
                                            image: port.clone(),
                                            ..default()
                                        },
                                        Node {
                                            position_type: PositionType::Absolute,
                                            left: px(left),
                                            top: px(top),
                                            width: px(PIPE_PORT_WIDTH),
                                            height: px(PIPE_TILE),
                                            ..default()
                                        },
                                        UiTransform::from_rotation(Rot2::degrees(quarters)),
                                    ));
                                }

                                for index in 0..PIPE_TILES {
                                    let at = pipe_tile_offset(index);

                                    canvas.spawn((
                                        PipeSlot::Coupling(index),
                                        ImageNode {
                                            image: straight.clone(),
                                            ..default()
                                        },
                                        Node {
                                            position_type: PositionType::Absolute,
                                            left: px(at.x),
                                            top: px(at.y),
                                            width: px(PIPE_TILE),
                                            height: px(PIPE_TILE),
                                            ..default()
                                        },
                                        UiTransform::default(),
                                    ));
                                }
                            });
                    }
                });
        });
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn run_active_minigame(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    settings: Res<Settings>,
    assets: Res<AssetServer>,
    mut commands: Commands,
    active: Option<ResMut<ActiveMinigame>>,
    mut status_labels: Query<&mut Text, With<MinigameStatus>>,
    // All `&mut Node` queries told apart only by their markers, which Bevy will
    // not take as disjoint — so they share a set rather than conflicting.
    mut visual_nodes: ParamSet<(
        Query<&mut Node, With<SequenceWireLeft>>,
        Query<&mut Node, With<SequenceWireRight>>,
        Query<&mut Node, With<SequenceWireSplitVisual>>,
        Query<&mut Node, With<SequenceWireJoint>>,
        Query<(&CoolantGaugeVisual, &mut Node)>,
        Query<(&EngineBoreVisual, &mut Node)>,
        Query<&mut Node, With<PipeWrench>>,
    )>,
    mut pipes: Query<(&PipeSlot, &mut ImageNode, &mut UiTransform)>,
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
        commands.remove_resource::<ActiveMinigame>();
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
            MinigameAudioCue::CoolantVent => {
                commands.spawn((
                    AudioPlayer::new(assets.load(GAUGE_HISS_PATH)),
                    PlaybackSettings::DESPAWN.with_volume(Volume::Linear(settings.sfx_volume)),
                ));
            }
            MinigameAudioCue::CoolantSealed => {
                commands.spawn((
                    AudioPlayer::new(assets.load(GAUGE_SEALED_TING_PATH)),
                    PlaybackSettings::DESPAWN.with_volume(Volume::Linear(settings.sfx_volume)),
                ));
            }
            // Borrowed from the coolant rig: both are a job coming good.
            MinigameAudioCue::PipesMadeUp => {
                commands.spawn((
                    AudioPlayer::new(assets.load(GAUGE_SEALED_TING_PATH)),
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

            for mut left in &mut visual_nodes.p0() {
                left.left = px(base_left - split_offset);
            }

            for mut right in &mut visual_nodes.p1() {
                right.left = px(base_right + split_offset);
            }

            for mut split in &mut visual_nodes.p2() {
                split.display = if visual.jointed {
                    Display::None
                } else {
                    Display::Flex
                };
            }

            for mut joint in &mut visual_nodes.p3() {
                joint.display = if visual.jointed {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        }
        MinigameVisualState::CoolantGauge(visual) => {
            let status = active.game.status();
            for mut text in &mut status_labels {
                **text = status.clone();
            }

            for (part, mut node) in &mut visual_nodes.p4() {
                match part {
                    // The two faces trade places on sealing: whichever is not
                    // being shown is the one hidden.
                    CoolantGaugeVisual::Face => {
                        node.display = if visual.sealed {
                            Display::None
                        } else {
                            Display::Flex
                        };
                    }
                    CoolantGaugeVisual::SealedFace => {
                        node.display = if visual.sealed {
                            Display::Flex
                        } else {
                            Display::None
                        };
                    }
                    CoolantGaugeVisual::Needle => {
                        node.left = px(needle_left(visual.fill));
                    }
                }
            }
        }
        MinigameVisualState::EngineBore(visual) => {
            let status = active.game.status();
            for mut text in &mut status_labels {
                **text = status.clone();
            }

            for (part, mut node) in &mut visual_nodes.p5() {
                match part {
                    // Widening the window is the whole animation: the clean
                    // plate is already sitting behind it, waiting to be let out.
                    EngineBoreVisual::Cut(row) => {
                        let cut = visual.cut[*row].min(BORE_CELLS) as f32;
                        node.width = px(cut * BORE_CELL);
                    }
                    EngineBoreVisual::Brush => {
                        node.top = px(visual.row as f32 * BORE_CELL);
                    }
                }
            }
        }
        MinigameVisualState::PipeRun(visual) => {
            let status = active.game.status();
            for mut text in &mut status_labels {
                **text = status.clone();
            }

            // The asset server hands back the handle it already loaded, so this
            // is a clone per piece rather than a load.
            let art = |flowing: bool, piece: PipePiece| {
                load_pixel_art(
                    &assets,
                    match (piece, flowing) {
                        (PipePiece::Straight, false) => PIPE_STRAIGHT_PATH,
                        (PipePiece::Straight, true) => PIPE_STRAIGHT_FLOW_PATH,
                        (PipePiece::Elbow, false) => PIPE_ELBOW_PATH,
                        (PipePiece::Elbow, true) => PIPE_ELBOW_FLOW_PATH,
                    },
                )
            };

            for (slot, mut image, mut transform) in &mut pipes {
                match *slot {
                    PipeSlot::Coupling(index) => {
                        let tile = visual.tiles[index];

                        image.image = art(tile.flowing, tile.piece);
                        transform.rotation = Rot2::degrees(90.0 * tile.quarters as f32);
                    }
                    // The inlet always runs; the drain only once the line is up.
                    PipeSlot::Inlet => {
                        image.image = load_pixel_art(&assets, PIPE_PORT_FLOW_PATH);
                    }
                    PipeSlot::Drain => {
                        image.image = load_pixel_art(
                            &assets,
                            if visual.drained {
                                PIPE_PORT_FLOW_PATH
                            } else {
                                PIPE_PORT_PATH
                            },
                        );
                    }
                }
            }

            let wrench = pipe_tile_offset(visual.cursor);
            for mut wash in &mut visual_nodes.p6() {
                wash.left = px(wrench.x);
                wash.top = px(wrench.y);
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
}

pub fn clear_active_minigame(mut commands: Commands) {
    commands.remove_resource::<ActiveMinigame>();
    commands.remove_resource::<PendingMinigame>();
    commands.remove_resource::<CompletedMinigame>();
}

fn new_minigame(id: MinigameId, keys: MinigameKeys) -> Box<dyn MinigameInstance> {
    match id {
        MinigameId::BrokenWire => Box::new(broken_wire::BrokenWire::new(keys)),
        MinigameId::CoolantValve => Box::new(coolant_valve::CoolantValve::new(keys)),
        // No art of its own yet: the default `visual_state` prints `status`
        // into the window's text, which is the whole of its display.
        MinigameId::CleanEngine => Box::new(clean_engine::CleanEngine::new(keys)),
        MinigameId::PipeFlow => Box::new(pipe_flow::PipeFlow::new(keys)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHALLENGE_ASSETS: [&str; 16] = [
        WIRES_BROKEN_PATH,
        WIRES_JOINT_PATH,
        WIRES_ZAP_PATH,
        GAUGE_PATH,
        GAUGE_SEALED_PATH,
        GAUGE_NEEDLE_PATH,
        GAUGE_HISS_PATH,
        GAUGE_SEALED_TING_PATH,
        BORE_CLEAN_PATH,
        BORE_DIRTY_PATH,
        PIPE_STRAIGHT_PATH,
        PIPE_STRAIGHT_FLOW_PATH,
        PIPE_ELBOW_PATH,
        PIPE_ELBOW_FLOW_PATH,
        PIPE_PORT_PATH,
        PIPE_PORT_FLOW_PATH,
    ];

    /// A renamed asset does not break the build — it just loads nothing, in a
    /// room the player has to walk to before they find out.
    #[test]
    fn every_challenge_asset_is_where_it_is_asked_for() {
        let assets = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");

        for path in CHALLENGE_ASSETS {
            assert!(
                assets.join(path).exists(),
                "no asset at assets/{path} — was it renamed?"
            );
        }
    }

    #[test]
    fn every_challenge_can_be_opened() {
        let keys = MinigameKeys {
            primary: KeyCode::KeyA,
            secondary: KeyCode::KeyD,
            up: KeyCode::KeyW,
            down: KeyCode::KeyS,
            action: KeyCode::Space,
        };

        for id in MinigameId::ALL {
            let game = new_minigame(id, keys);

            assert!(!game.title().is_empty(), "{id:?} has no title");
            assert!(!game.status().is_empty(), "{id:?} has no status");
        }
    }

    #[test]
    fn the_needle_stays_inside_the_gauge_canvas() {
        for fill in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let left = needle_left(fill);

            assert!(left >= 0.0, "the needle hangs off the empty end at {fill}");
            assert!(
                left + GAUGE_NEEDLE_WIDTH <= GAUGE_CANVAS_WIDTH,
                "the needle hangs off the stop at {fill}"
            );
        }
    }

    #[test]
    fn the_needle_travels_the_painted_track() {
        let centre = |fill: f32| needle_left(fill) + GAUGE_NEEDLE_WIDTH * 0.5;

        assert!((centre(0.0) - GAUGE_TRACK_LEFT * GAUGE_WIDTH).abs() < 0.5);
        assert!(
            (centre(1.0) - (GAUGE_TRACK_LEFT + GAUGE_TRACK_WIDTH) * GAUGE_WIDTH).abs() < 0.5,
            "the needle does not reach the end of the track"
        );
    }

    #[test]
    fn an_out_of_range_reading_is_pinned_to_the_gauge() {
        assert_eq!(needle_left(-3.0), needle_left(0.0));
        assert_eq!(needle_left(9.0), needle_left(1.0));
    }
}
