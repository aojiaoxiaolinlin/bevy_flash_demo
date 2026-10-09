//! Native Bevy UI controls shared by desktop and WebGPU builds.
use crate::{catalog, demos::Demo};
use bevy::{
    camera::{CameraOutputMode, Viewport, visibility::RenderLayers},
    input::mouse::{AccumulatedMouseMotion, MouseWheel},
    picking::hover::Hovered,
    prelude::*,
    render::render_resource::BlendState,
    ui::{IsDefaultUiCamera, Pressed},
    ui_widgets::{Activate, Button},
};

pub const PANEL_WIDTH: f32 = 320.0;
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum Page {
    #[default]
    Animation,
    Skins,
    Ui,
    Buttons,
}
#[derive(Message, Component, Clone)]
pub enum Action {
    Navigate { page: Page, index: usize },
    SelectPage { page: Page },
    Play { name: String },
    Fallback { name: String },
    Skin { slot: String, variant: String },
    Pause,
    Zoom { factor: f32 },
    Fit,
    Retry,
}
#[derive(Default)]
pub struct Status {
    pub page: Page,
    pub index: usize,
    pub ready: bool,
    pub error: Option<String>,
    pub notice: String,
    pub clips: Vec<String>,
    pub skins: Vec<(String, Vec<String>, String)>,
    pub clip: String,
    pub fallback: String,
    pub playing: bool,
    pub zoom: f32,
}
#[derive(Component)]
pub struct WorldCamera;
#[derive(Component)]
pub(crate) struct Panel;
#[derive(Component)]
pub(crate) struct ScrollArea;
#[derive(Component)]
pub(crate) struct Info;
#[derive(Component)]
pub(crate) struct ControlLabel;
#[derive(Resource, Default)]
pub(crate) struct PanelKey(Option<(Page, usize, bool)>);

pub fn setup(mut commands: Commands) {
    commands.init_resource::<PanelKey>();
    commands.spawn((WorldCamera, Camera2d, CompositingSpace::Srgb, Msaa::Sample4));
    // UI shaders output linear color; keep their camera separate from Flash compositing.
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            clear_color: ClearColorConfig::Custom(Color::NONE),
            output_mode: CameraOutputMode::Write {
                blend_state: Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                clear_color: ClearColorConfig::None,
            },
            ..default()
        },
        IsDefaultUiCamera,
        RenderLayers::layer(31),
    ));
    commands.spawn((
        Panel,
        Node {
            position_type: PositionType::Absolute,
            width: px(PANEL_WIDTH),
            top: px(0),
            bottom: px(0),
            padding: UiRect::all(px(12)),
            row_gap: px(8),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgb(0.055, 0.065, 0.085)),
    ));
}
pub fn activate(event: On<Activate>, controls: Query<&Action>, mut actions: MessageWriter<Action>) {
    if let Ok(action) = controls.get(event.entity) {
        actions.write(action.clone());
    }
}

const MUTED: Color = Color::srgb(0.57, 0.63, 0.72);
const NORMAL: Color = Color::srgb(0.12, 0.15, 0.20);
const ACTIVE: Color = Color::srgb(0.18, 0.29, 0.43);
const PLAYING: Color = Color::srgb(0.40, 0.92, 0.64);
const FALLBACK: Color = Color::srgb(0.40, 0.75, 1.0);

fn caption(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    size: f32,
    color: Color,
    font: &Handle<Font>,
) {
    parent.spawn((
        Text::new(label),
        TextFont {
            font: font.clone().into(),
            font_size: size.into(),
            ..default()
        },
        TextColor(color),
        Node {
            flex_shrink: 0.0,
            ..default()
        },
    ));
}

fn button(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    action: Action,
    font: &Handle<Font>,
    grow: f32,
) {
    let width = if matches!(&action, Action::Fallback { .. }) {
        px(105)
    } else {
        auto()
    };
    parent
        .spawn((
            Button,
            Hovered::default(),
            action,
            Node {
                width,
                min_height: px(32),
                padding: UiRect::axes(px(9), px(6)),
                flex_grow: grow,
                flex_basis: if grow > 0.0 { px(0) } else { auto() },
                flex_shrink: 0.0,
                align_items: AlignItems::Center,
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(6)),
                ..default()
            },
            BackgroundColor(NORMAL),
            BorderColor::all(Color::srgb(0.19, 0.23, 0.30)),
        ))
        .with_children(|row| {
            row.spawn((
                ControlLabel,
                Text::new(label),
                TextFont {
                    font: font.clone().into(),
                    font_size: 14.0.into(),
                    ..default()
                },
                TextColor(Color::srgb(0.85, 0.88, 0.93)),
            ));
        });
}

fn row() -> Node {
    Node {
        column_gap: px(6),
        flex_shrink: 0.0,
        ..default()
    }
}

pub fn panel(
    mut commands: Commands,
    demo: Res<Demo>,
    mut key: ResMut<PanelKey>,
    panel: Query<(Entity, Option<&Children>), With<Panel>>,
    server: Option<Res<AssetServer>>,
) {
    let next = (demo.status.page, demo.status.index, demo.status.ready);
    if key.0 == Some(next) {
        return;
    }
    key.0 = Some(next);
    let Ok((entity, children)) = panel.single() else {
        return;
    };
    if let Some(children) = children {
        for child in children.iter() {
            commands.entity(child).despawn();
        }
    }
    let font = server
        .map(|s| s.load("fonts/FiraSans-Bold.ttf"))
        .unwrap_or_default();
    commands.entity(entity).with_children(|parent| {
        caption(parent, "bevy_flash", 23.0, Color::WHITE, &font);
        caption(parent, "VECTOR ANIMATION / WEBGPU", 11.0, MUTED, &font);
        parent.spawn(row()).with_children(|tabs| {
            for (label, page) in [
                ("Anim", Page::Animation),
                ("Skins", Page::Skins),
                ("UI", Page::Ui),
                ("Buttons", Page::Buttons),
            ] {
                button(tabs, label, Action::SelectPage { page }, &font, 1.0);
            }
        });
        caption(parent, "RESOURCE", 11.0, MUTED, &font);
        let entries = match demo.status.page {
            Page::Animation => catalog::ANIMATIONS,
            Page::Ui => catalog::GRAPHICS,
            _ => &[],
        };
        parent.spawn(row()).with_children(|resources| {
            if entries.is_empty() {
                caption(
                    resources,
                    if demo.status.page == Page::Skins {
                        "Wu Kong / named skins"
                    } else {
                        "Login / Flash button"
                    },
                    14.0,
                    Color::WHITE,
                    &font,
                );
            } else {
                let current = demo.status.index.min(entries.len() - 1);
                button(
                    resources,
                    "<",
                    Action::Navigate {
                        page: demo.status.page,
                        index: (current + entries.len() - 1) % entries.len(),
                    },
                    &font,
                    0.0,
                );
                resources
                    .spawn(Node {
                        flex_grow: 1.0,
                        flex_basis: px(0),
                        min_width: px(0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    })
                    .with_children(|label| {
                        caption(label, entries[current].0, 14.0, Color::WHITE, &font)
                    });
                button(
                    resources,
                    ">",
                    Action::Navigate {
                        page: demo.status.page,
                        index: (current + 1) % entries.len(),
                    },
                    &font,
                    0.0,
                );
            }
        });
        caption(
            parent,
            if demo.status.clips.is_empty() {
                "PREVIEW"
            } else {
                "ACTIONS"
            },
            11.0,
            MUTED,
            &font,
        );
        // Only the action/skin region scrolls; view controls remain reachable.
        parent
            .spawn((
                ScrollArea,
                ScrollPosition::default(),
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: px(5),
                    flex_grow: 1.0,
                    flex_basis: px(0),
                    min_height: px(0),
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
            ))
            .with_children(|list| {
                if !demo.status.ready {
                    caption(list, "Loading...", 14.0, MUTED, &font);
                }
                for name in &demo.status.clips {
                    list.spawn(row()).with_children(|row| {
                        button(row, name, Action::Play { name: name.clone() }, &font, 1.0);
                        button(
                            row,
                            "Set fallback",
                            Action::Fallback { name: name.clone() },
                            &font,
                            0.0,
                        );
                    });
                }
                for (slot, variants, _) in &demo.status.skins {
                    if let Some(variant) = variants.first() {
                        button(
                            list,
                            slot,
                            Action::Skin {
                                slot: slot.clone(),
                                variant: variant.clone(),
                            },
                            &font,
                            0.0,
                        );
                    }
                }
                if demo.status.ready && demo.status.clips.is_empty() {
                    caption(list, &demo.status.notice, 13.0, MUTED, &font);
                }
            });
        // Footer stays outside the scrolling list.
        parent
            .spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: px(7),
                    flex_shrink: 0.0,
                    padding: UiRect::top(px(10)),
                    border: UiRect::top(px(1)),
                    ..default()
                },
                BorderColor::all(Color::srgb(0.19, 0.23, 0.30)),
            ))
            .with_children(|footer| {
                footer.spawn((
                    Info,
                    Text::new("Loading..."),
                    TextFont {
                        font: font.clone().into(),
                        font_size: 13.0.into(),
                        ..default()
                    },
                    TextColor(Color::srgb(0.78, 0.83, 0.91)),
                    Node {
                        flex_shrink: 0.0,
                        ..default()
                    },
                ));
                if demo.status.ready {
                    footer.spawn(row()).with_children(|row| {
                        button(row, "Pause", Action::Pause, &font, 1.0);
                        button(row, "Fit view", Action::Fit, &font, 1.0);
                    });
                    footer.spawn(row()).with_children(|row| {
                        button(row, "Zoom -", Action::Zoom { factor: 0.8 }, &font, 1.0);
                        button(row, "Zoom +", Action::Zoom { factor: 1.25 }, &font, 1.0);
                    });
                }
                caption(
                    footer,
                    "Right drag / arrows: pan\nWheel: zoom   Space: pause",
                    11.0,
                    MUTED,
                    &font,
                );
                button(footer, "Reload resource", Action::Retry, &font, 0.0);
            });
    });
}

#[allow(clippy::type_complexity)]
pub fn status_text(
    demo: Res<Demo>,
    mut info: Query<&mut Text, With<Info>>,
    mut labels: Query<(&mut Text, &mut TextColor, &ChildOf), (With<ControlLabel>, Without<Info>)>,
    controls: Query<&Action>,
    mut buttons: Query<(
        &Action,
        &Hovered,
        Has<Pressed>,
        &mut BackgroundColor,
        &mut BorderColor,
    )>,
) {
    let status = &demo.status;
    for (action, hovered, pressed, mut background, mut border) in &mut buttons {
        let selected = match action {
            Action::SelectPage { page } => *page == status.page,
            Action::Navigate { page, index } => *page == status.page && *index == status.index,
            Action::Play { name } => *name == status.clip,
            Action::Fallback { name } => *name == status.fallback,
            _ => false,
        };
        let color = if pressed {
            Color::srgb(0.27, 0.40, 0.55)
        } else if selected {
            ACTIVE
        } else if hovered.0 {
            Color::srgb(0.18, 0.22, 0.29)
        } else {
            NORMAL
        };
        if background.0 != color {
            background.0 = color;
        }
        let edge = if selected {
            FALLBACK
        } else {
            Color::srgb(0.19, 0.23, 0.30)
        };
        if border.top != edge {
            *border = BorderColor::all(edge);
        }
    }
    for mut text in &mut info {
        let value = if let Some(error) = &status.error {
            error.clone()
        } else if !status.ready {
            "Loading resource...".into()
        } else if !status.clips.is_empty() {
            format!(
                "{}: {}\nFallback: {}\nZoom: {:.0}%",
                if status.playing { "Playing" } else { "Paused" },
                status.clip,
                status.fallback,
                status.zoom * 100.0
            )
        } else {
            format!(
                "{} / Zoom: {:.0}%",
                if status.playing { "Playing" } else { "Paused" },
                status.zoom * 100.0
            )
        };
        if text.0 != value {
            text.0 = value;
        }
    }
    for (mut text, mut color, parent) in &mut labels {
        let Ok(action) = controls.get(parent.parent()) else {
            continue;
        };
        let (value, wanted) = match action {
            Action::Play { name } => (
                if *name == status.clip {
                    format!("{name}  >")
                } else {
                    name.clone()
                },
                if *name == status.clip {
                    PLAYING
                } else {
                    Color::srgb(0.85, 0.88, 0.93)
                },
            ),
            Action::Fallback { name } => (
                if *name == status.fallback {
                    "Fallback"
                } else {
                    "Set fallback"
                }
                .to_string(),
                if *name == status.fallback {
                    FALLBACK
                } else {
                    MUTED
                },
            ),
            Action::Pause => (
                if status.playing { "Pause" } else { "Resume" }.to_string(),
                Color::WHITE,
            ),
            Action::Skin { slot, .. } => (
                status.skins.iter().find(|(s, _, _)| s == slot).map_or_else(
                    || slot.clone(),
                    |(_, _, selected)| format!("{slot}: {selected} / next"),
                ),
                Color::WHITE,
            ),
            _ => continue,
        };
        if text.0 != value {
            text.0 = value;
        }
        if color.0 != wanted {
            color.0 = wanted;
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn camera_input(
    windows: Query<&Window>,
    mut cameras: Query<(&mut Camera, &mut Transform), With<WorldCamera>>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    mut wheel: MessageReader<MouseWheel>,
    mut panels: Query<&mut ScrollPosition, With<ScrollArea>>,
    mut actions: MessageWriter<Action>,
    time: Res<Time>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let in_panel = window.cursor_position().is_some_and(|p| p.x < PANEL_WIDTH);
    let physical = window.physical_size();
    let left = ((PANEL_WIDTH * window.scale_factor()) as u32).min(physical.x.saturating_sub(1));
    for (mut camera, mut transform) in &mut cameras {
        camera.viewport = Some(Viewport {
            physical_position: UVec2::new(left, 0),
            physical_size: UVec2::new((physical.x - left).max(1), physical.y.max(1)),
            ..default()
        });
        let axis = Vec2::new(
            (keys.pressed(KeyCode::ArrowRight) as i8 - keys.pressed(KeyCode::ArrowLeft) as i8)
                as f32,
            (keys.pressed(KeyCode::ArrowUp) as i8 - keys.pressed(KeyCode::ArrowDown) as i8) as f32,
        );
        transform.translation += (axis * time.delta_secs() * 400.0).extend(0.0);
        if !in_panel && mouse.pressed(MouseButton::Right) {
            transform.translation += Vec3::new(-motion.delta.x, motion.delta.y, 0.0);
        }
    }
    for event in wheel.read() {
        if in_panel {
            for mut scroll in &mut panels {
                scroll.0.y = (scroll.0.y - event.y * 24.0).max(0.0);
            }
        } else {
            actions.write(Action::Zoom {
                factor: (event.y * 0.1).exp(),
            });
        }
    }
}

#[cfg(test)]
mod tests;
