//! Four pages share one app/camera. Only the active page owns scene handles.
use crate::{
    bounds,
    bridge::{self, Action, Page, Status},
    catalog,
};
use bevy::{asset::LoadState, prelude::*, ui::InteractionDisabled};
use bevy_flash::{
    sampling::VabSkin,
    vab_asset::{VabAsset, VabAssetHandle},
    vab_button::VabButton,
    vab_graphic::VabGraphic,
    vab_player::{VabCompleteEvent, VabFrameEvent, VabPlayer},
    vab_ui::{VabButtonNode, VabImageNode, VabUiPlayback},
};

#[derive(Component)]
struct SceneRoot;
#[derive(Component)]
struct Actor;
#[derive(Component)]
struct PresentationWidth(f32);
#[derive(Component)]
struct Framing {
    center: Vec2,
    size: Vec2,
}
#[derive(Clone, Default)]
enum Target {
    #[default]
    None,
    Animation(Handle<VabAsset>),
    Graphic(Handle<VabGraphic>),
    Button(Handle<VabButton>),
}
#[derive(Resource)]
struct Demo {
    status: Status,
    target: Target,
    dirty: bool,
    diagnostics: bool,
    smoothed_fps: f32,
}
impl Default for Demo {
    fn default() -> Self {
        Self {
            status: Status::default(),
            target: Target::None,
            dirty: true,
            diagnostics: false,
            smoothed_fps: 0.0,
        }
    }
}
impl Demo {
    fn event(&mut self, text: String) {
        self.status.events.push(text);
        if self.status.events.len() > 8 {
            self.status.events.remove(0);
        }
    }
    fn result(&mut self, result: Result<(), String>) {
        if let Err(error) = result {
            self.event(error);
        }
    }
}

pub struct DemoPlugin;
impl Plugin for DemoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Demo>()
            .add_systems(Startup, |mut commands: Commands| {
                commands.spawn((Camera2d, CompositingSpace::Srgb, Msaa::Sample4));
            })
            .add_systems(
                Update,
                (
                    keyboard,
                    actions,
                    rebuild,
                    ready,
                    fit,
                    button_clicks,
                    events,
                    publish,
                )
                    .chain(),
            );
    }
}

fn keyboard(keys: Res<ButtonInput<KeyCode>>) {
    for (key, page) in [
        (KeyCode::Digit1, Page::Animation),
        (KeyCode::Digit2, Page::Skins),
        (KeyCode::Digit3, Page::Ui),
        (KeyCode::Digit4, Page::Buttons),
    ] {
        if keys.just_pressed(key) {
            bridge::push(Action::Navigate { page, index: 0 });
        }
    }
    if keys.just_pressed(KeyCode::Space) {
        bridge::push(Action::Pause);
    }
}

#[allow(clippy::too_many_arguments)]
fn actions(
    mut commands: Commands,
    mut demo: ResMut<Demo>,
    assets: Res<Assets<VabAsset>>,
    mut actor: Query<(&VabAssetHandle, &mut VabPlayer, &mut VabSkin), With<Actor>>,
    mut ui: Query<&mut VabUiPlayback>,
    buttons: Query<Entity, With<VabButtonNode>>,
    server: Res<AssetServer>,
) {
    for action in bridge::drain() {
        match action {
            Action::Navigate { page, index } => {
                let count = match page {
                    Page::Animation => catalog::ANIMATIONS.len(),
                    Page::Ui => catalog::GRAPHICS.len(),
                    _ => 1,
                };
                demo.status.page = page;
                demo.status.index = index.min(count - 1);
                demo.dirty = true;
            }
            Action::Retry => {
                let path = match demo.status.page {
                    Page::Animation => catalog::ANIMATIONS[demo.status.index].1,
                    Page::Skins => catalog::SKIN_ASSET,
                    Page::Ui => catalog::GRAPHICS[demo.status.index].1,
                    Page::Buttons => catalog::BUTTON_ASSET,
                };
                server.reload(path.split('#').next().unwrap_or(path));
                demo.dirty = true;
            }
            Action::Diagnostics { value } => demo.diagnostics = value,
            Action::Zoom { value } => {
                if value.is_finite() {
                    demo.status.zoom = value.clamp(0.25, 4.0);
                }
            }
            Action::Disabled { value } => {
                for entity in &buttons {
                    if value {
                        commands.entity(entity).insert(InteractionDisabled);
                    } else {
                        commands.entity(entity).remove::<InteractionDisabled>();
                    }
                }
            }
            Action::Pause => {
                for (_, mut player, _) in &mut actor {
                    if player.playing {
                        player.pause()
                    } else {
                        player.resume()
                    }
                }
                for mut player in &mut ui {
                    if player.playing {
                        player.pause()
                    } else {
                        player.resume()
                    }
                }
            }
            Action::Speed { value } => {
                let value = value.clamp(0.25, 4.0);
                for (_, mut player, _) in &mut actor {
                    demo.result(player.set_speed(value).map_err(|e| e.to_string()));
                }
                for mut player in &mut ui {
                    demo.result(player.set_speed(value).map_err(|e| e.to_string()));
                }
            }
            action => {
                if !demo.status.ready || demo.dirty {
                    continue;
                }
                for (handle, mut player, mut skin) in &mut actor {
                    let Some(asset) = assets.get(&handle.0) else {
                        continue;
                    };
                    let result = match &action {
                        Action::Play { name, mode } => match mode.as_str() {
                            "loop" => player.play_loop(asset, name).map(|_| ()),
                            "once" => player.play_once(asset, name).map(|_| ()),
                            "hold" => player.play_once_and_hold(asset, name).map(|_| ()),
                            "terminal" => player.play_terminal(asset, name).map(|_| ()),
                            "chain" => {
                                let fallback = demo.status.fallback.clone();
                                player
                                    .play_once(asset, name)
                                    .and_then(|p| p.then_once(asset, name))
                                    .and_then(|p| p.then_loop(asset, &fallback))
                                    .map(|_| ())
                            }
                            _ => continue,
                        },
                        Action::Fallback { name } => player
                            .set_fallback_loop(asset, name)
                            .map(|_| ())
                            .inspect(|_| demo.status.fallback = name.clone()),
                        Action::Seek { frame } => player.seek(asset, *frame),
                        Action::Skin { slot, variant } => skin.set(asset, slot, variant),
                        Action::Reset => {
                            player.reset_terminal();
                            player.play_loop(asset, &demo.status.fallback).map(|_| ())
                        }
                        _ => continue,
                    };
                    demo.result(result.map_err(|e| e.to_string()));
                }
            }
        }
    }
}

fn rebuild(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut demo: ResMut<Demo>,
    old: Query<Entity, With<SceneRoot>>,
) {
    if !demo.dirty {
        return;
    }
    for entity in &old {
        commands.entity(entity).despawn();
    }
    let page = demo.status.page;
    let index = demo.status.index;
    demo.status = Status {
        page,
        index,
        notice: "正在加载资源与依赖…".into(),
        speed: 1.0,
        zoom: 1.0,
        ..default()
    };
    demo.target = match page {
        Page::Animation | Page::Skins => {
            let path = if page == Page::Skins {
                catalog::SKIN_ASSET
            } else {
                catalog::ANIMATIONS[index].1
            };
            let handle: Handle<VabAsset> = server.load(path);
            commands.spawn((
                SceneRoot,
                Actor,
                VabAssetHandle(handle.clone()),
                VabPlayer::default(),
                VabSkin::default(),
                Transform::default(),
                Visibility::Hidden,
            ));
            demo.status.assets = if page == Page::Skins {
                vec!["悟空 · 命名皮肤".into()]
            } else {
                catalog::ANIMATIONS.iter().map(|a| a.0.into()).collect()
            };
            Target::Animation(handle)
        }
        Page::Ui => {
            demo.status.assets = catalog::GRAPHICS.iter().map(|a| a.0.into()).collect();
            Target::Graphic(server.load(catalog::GRAPHICS[index].1))
        }
        Page::Buttons => {
            demo.status.assets = vec!["登录 · DefineButton2".into()];
            Target::Button(server.load(catalog::BUTTON_ASSET))
        }
    };
    demo.dirty = false;
}

fn load_error(server: &AssetServer, id: impl Into<bevy::asset::UntypedAssetId>) -> Option<String> {
    let id = id.into();
    if let LoadState::Failed(error) = server.load_state(id) {
        return Some(error.to_string());
    }
    if let Some(bevy::asset::RecursiveDependencyLoadState::Failed(error)) =
        server.get_recursive_dependency_load_state(id)
    {
        return Some(error.to_string());
    }
    None
}

#[allow(clippy::too_many_arguments)]
fn ready(
    mut commands: Commands,
    server: Res<AssetServer>,
    assets: Res<Assets<VabAsset>>,
    graphics: Res<Assets<VabGraphic>>,
    buttons: Res<Assets<VabButton>>,
    mut demo: ResMut<Demo>,
    mut actors: Query<(Entity, &mut VabPlayer, &mut VabSkin), With<Actor>>,
) {
    if demo.status.ready || demo.status.error.is_some() {
        return;
    }
    let error = match &demo.target {
        Target::Animation(h) => load_error(&server, h.id()),
        Target::Graphic(h) => load_error(&server, h.id()),
        Target::Button(h) => load_error(&server, h.id()),
        Target::None => None,
    };
    if let Some(error) = error {
        demo.status.error = Some(error);
        return;
    }
    let target = demo.target.clone();
    match &target {
        Target::Animation(handle) => {
            if !server.is_loaded_with_dependencies(handle.id()) {
                return;
            }
            let Some(asset) = assets.get(handle) else {
                return;
            };
            let Some(initial) = asset
                .baked
                .clips
                .iter()
                .find(|c| c.name.eq_ignore_ascii_case("idle"))
                .or_else(|| asset.baked.clips.first())
            else {
                demo.status.error = Some("资源没有动作".into());
                return;
            };
            let initial_name = initial.name.clone();
            demo.status.clips = asset.baked.clips.iter().map(|c| c.name.clone()).collect();
            demo.status.fallback = initial_name.clone();
            for (entity, mut player, mut skin) in &mut actors {
                for slot in asset.skin_slots() {
                    match asset.skin_variant_names(&slot) {
                        Ok(variants) => {
                            if let Some(name) = variants.first() {
                                let _ = skin.set(asset, &slot, name);
                            }
                        }
                        Err(error) => {
                            demo.status.error = Some(error.to_string());
                            return;
                        }
                    }
                }
                if let Err(error) = player
                    .set_fallback_loop(asset, &initial_name)
                    .and_then(|p| p.play_loop(asset, &initial_name))
                {
                    demo.status.error = Some(error.to_string());
                    return;
                }
                match bounds::measure(asset, &skin) {
                    Ok((center, size)) => {
                        commands
                            .entity(entity)
                            .insert((Framing { center, size }, Visibility::Visible));
                    }
                    Err(error) => {
                        demo.status.error = Some(error);
                        return;
                    }
                }
            }
            demo.status.notice = "动作结束自动回到指定 fallback；终态播放需要重置才能切换。".into();
        }
        Target::Graphic(handle) => {
            if !server.is_loaded_with_dependencies(handle.id()) {
                return;
            }
            let Some(graphic) = graphics.get(handle) else {
                return;
            };
            let handle = handle.clone();
            let animated = graphic.is_animated();
            let widths = if animated {
                [170.0, 170.0, 280.0]
            } else {
                [220.0, 220.0, 420.0]
            };
            commands
                .spawn((
                    SceneRoot,
                    Node {
                        width: percent(100),
                        height: percent(100),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        flex_direction: FlexDirection::Column,
                        row_gap: px(24),
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    for width in widths {
                        parent.spawn((
                            VabImageNode::new(handle.clone()),
                            PresentationWidth(width),
                            Node {
                                width: px(width),
                                max_width: percent(90),
                                ..default()
                            },
                        ));
                    }
                });
            demo.status.frames = graphic.frame_count;
            demo.status.notice = format!(
                "{} 帧 · 两个同尺寸节点共享栅格缓存；大节点独立渲染，默认等比缩放。",
                graphic.frame_count
            );
        }
        Target::Button(handle) => {
            if !server.is_loaded_with_dependencies(handle.id()) || !buttons.contains(handle.id()) {
                return;
            }
            let handle = handle.clone();
            commands
                .spawn((
                    SceneRoot,
                    Node {
                        width: percent(100),
                        height: percent(100),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        flex_direction: FlexDirection::Column,
                        row_gap: px(32),
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    for width in [180.0, 180.0, 300.0] {
                        parent.spawn((
                            VabButtonNode::new(handle.clone()),
                            Node {
                                width: px(width),
                                max_width: percent(90),
                                ..default()
                            },
                        ));
                    }
                });
            demo.status.notice =
                "指针悬停显示 over，按下显示 down；hit 是命中几何，当前使用 UI 矩形交互。".into();
        }
        Target::None => return,
    }
    demo.status.ready = true;
}

fn fit(
    windows: Query<&Window>,
    mut demo: ResMut<Demo>,
    mut actors: Query<(&Framing, &mut Transform), With<Actor>>,
    graphics: Res<Assets<VabGraphic>>,
    mut images: Query<(&PresentationWidth, &VabImageNode, &mut Node)>,
) {
    let Ok(window) = windows.single() else { return };
    for (frame, mut transform) in &mut actors {
        let scale = (window.width() * 0.88 / frame.size.x)
            .min(window.height() * 0.82 / frame.size.y)
            .clamp(0.01, 8.0)
            * demo.status.zoom;
        // Zoom around the fixed framing center, never the current frame bounds.
        demo.status.render_scale = scale;
        transform.scale = Vec3::splat(scale);
        transform.translation = Vec3::new(-frame.center.x * scale, frame.center.y * scale, 0.0);
    }
    let mut total_height = 0.0_f32;
    let mut widest = 1.0_f32;
    for (width, image, _) in &images {
        if let Some(graphic) = graphics.get(&image.graphic) {
            let size = graphic.visual_size();
            total_height += width.0 * size.y / size.x.max(1.0);
            widest = widest.max(width.0);
        }
    }
    let factor = ((window.height() * 0.88 - 48.0) / total_height.max(1.0))
        .min(window.width() * 0.90 / widest)
        .clamp(0.05, 1.0);
    for (width, _, mut node) in &mut images {
        let desired = px(width.0 * factor);
        if node.width != desired {
            node.width = desired;
        }
    }
}

#[allow(clippy::type_complexity)]
fn button_clicks(
    mut demo: ResMut<Demo>,
    buttons: Query<
        (&Interaction, Has<InteractionDisabled>),
        (With<VabButtonNode>, Changed<Interaction>),
    >,
) {
    for (interaction, disabled) in &buttons {
        if !disabled && *interaction == Interaction::Pressed {
            demo.status.clicks += 1;
            demo.event("登录按钮被按下".into());
        }
    }
}

fn events(
    mut demo: ResMut<Demo>,
    mut frames: MessageReader<VabFrameEvent>,
    mut ends: MessageReader<VabCompleteEvent>,
    actors: Query<(), With<Actor>>,
) {
    for event in frames.read() {
        if actors.contains(event.entity) {
            demo.event(format!(
                "{} · 帧 {} · {}",
                event.animation,
                event.frame + 1,
                event.name
            ));
        }
    }
    for event in ends.read() {
        if actors.contains(event.entity) {
            demo.event(format!("{} · 播放完成", event.animation));
        }
    }
}

fn publish(
    mut demo: ResMut<Demo>,
    assets: Res<Assets<VabAsset>>,
    time: Res<Time>,
    actor: Query<(&VabAssetHandle, &VabPlayer, &VabSkin), With<Actor>>,
    ui: Query<&VabUiPlayback>,
) {
    if let Ok((handle, player, skin)) = actor.single()
        && let Some(asset) = assets.get(&handle.0)
    {
        if let Some(clip) = asset.baked.clips.get(player.clip()) {
            demo.status.clip = clip.name.clone();
            demo.status.frames = clip.frames.len();
        }
        demo.status.frame = player.current_frame;
        demo.status.playing = player.playing;
        demo.status.speed = player.speed();
        demo.status.terminal = player.is_terminal();
        demo.status.skins = asset
            .skin_slots()
            .into_iter()
            .map(|slot| {
                let variants = asset.skin_variant_names(&slot).unwrap_or_default();
                let selected = skin.selection(&slot).unwrap_or("").to_owned();
                (slot, variants, selected)
            })
            .collect();
    } else if let Some(player) = ui.iter().next() {
        demo.status.frame = player.current_frame;
        demo.status.playing = player.playing;
        demo.status.speed = player.speed();
    }
    let seconds = time.delta_secs();
    if seconds > 0.0 {
        demo.smoothed_fps = demo.smoothed_fps * 0.9 + (1.0 / seconds) * 0.1;
    }
    demo.status.fps = demo.diagnostics.then_some(demo.smoothed_fps);
    bridge::publish(&demo.status);
}
