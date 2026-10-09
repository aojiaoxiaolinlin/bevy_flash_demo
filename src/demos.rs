//! Four pages share one app; only the active page owns scene handles.
use crate::{
    bounds, catalog,
    controls::{Action, PANEL_WIDTH, Page, Status, WorldCamera},
};
use bevy::{asset::LoadState, prelude::*};
use bevy_flash::{
    sampling::VabSkin,
    vab_asset::{VabAsset, VabAssetHandle},
    vab_button::VabButton,
    vab_graphic::VabGraphic,
    vab_player::VabPlayer,
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
pub(crate) struct Demo {
    pub(crate) status: Status,
    target: Target,
    dirty: bool,
}
impl Default for Demo {
    fn default() -> Self {
        Self {
            status: Status::default(),
            target: Target::None,
            dirty: true,
        }
    }
}
impl Demo {
    fn result(&mut self, result: Result<(), String>) {
        if let Err(error) = result {
            self.status.error = Some(error);
        }
    }
}

pub struct DemoPlugin;
impl Plugin for DemoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Demo>()
            .add_message::<Action>()
            .add_observer(crate::controls::activate)
            .add_systems(Startup, crate::controls::setup)
            .add_systems(
                Update,
                (
                    keyboard,
                    actions,
                    rebuild,
                    ready,
                    refresh_status,
                    crate::controls::panel,
                    crate::controls::camera_input,
                    fit,
                    crate::controls::status_text,
                )
                    .chain(),
            );
    }
}

fn keyboard(keys: Res<ButtonInput<KeyCode>>, mut actions: MessageWriter<Action>) {
    for (key, page) in [
        (KeyCode::Digit1, Page::Animation),
        (KeyCode::Digit2, Page::Skins),
        (KeyCode::Digit3, Page::Ui),
        (KeyCode::Digit4, Page::Buttons),
    ] {
        if keys.just_pressed(key) {
            actions.write(Action::Navigate { page, index: 0 });
        }
    }
    if keys.just_pressed(KeyCode::Space) {
        actions.write(Action::Pause);
    }
}

#[allow(clippy::too_many_arguments)]
fn actions(
    mut demo: ResMut<Demo>,
    assets: Res<Assets<VabAsset>>,
    mut actor: Query<(&VabAssetHandle, &mut VabPlayer, &mut VabSkin), With<Actor>>,
    mut ui: Query<&mut VabUiPlayback>,
    server: Res<AssetServer>,
    mut actions: MessageReader<Action>,
    mut cameras: Query<&mut Transform, With<WorldCamera>>,
) {
    for action in actions.read().cloned() {
        let action = match action {
            Action::SelectPage { page } => Action::Navigate {
                page,
                index: if page == demo.status.page {
                    demo.status.index
                } else {
                    0
                },
            },
            other => other,
        };
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
                for mut transform in &mut cameras {
                    transform.translation = Vec3::ZERO;
                }
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
            Action::Zoom { factor } => {
                demo.status.zoom = (demo.status.zoom * factor).clamp(0.25, 4.0)
            }
            Action::Fit => {
                demo.status.zoom = 1.0;
                for mut transform in &mut cameras {
                    transform.translation = Vec3::ZERO;
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
            action => {
                if !demo.status.ready || demo.dirty {
                    continue;
                }
                for (handle, mut player, mut skin) in &mut actor {
                    let Some(asset) = assets.get(&handle.0) else {
                        continue;
                    };
                    let result = match &action {
                        Action::Play { name } => player.play_once(asset, name).map(|_| ()),
                        Action::Fallback { name } => {
                            let result = player.set_fallback_loop(asset, name).map(|_| ());
                            if result.is_ok() {
                                demo.status.fallback = name.clone();
                            }
                            result
                        }
                        Action::Skin { slot, variant } => {
                            let variants = asset.skin_variant_names(slot).unwrap_or_default();
                            let next = variants
                                .iter()
                                .position(|v| Some(v.as_str()) == skin.selection(slot))
                                .and_then(|i| variants.get((i + 1) % variants.len()));
                            skin.set(asset, slot, next.unwrap_or(variant))
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
        notice: "Loading asset...".into(),
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
            Target::Animation(handle)
        }
        Page::Ui => Target::Graphic(server.load(catalog::GRAPHICS[index].1)),
        Page::Buttons => Target::Button(server.load(catalog::BUTTON_ASSET)),
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
                demo.status.error = Some("No animation clips".into());
                return;
            };
            let initial_name = initial.name.clone();
            demo.status.fallback = initial_name.clone();
            demo.status.clips = asset.baked.clips.iter().map(|c| c.name.clone()).collect();
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
            demo.status.notice = "Click an action to play. Blue marks its fallback.".into();
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
                        position_type: PositionType::Absolute,
                        left: px(PANEL_WIDTH),
                        right: px(0),
                        top: px(0),
                        bottom: px(0),
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
            demo.status.notice = format!(
                "{} · aspect-preserving vector UI",
                if animated { "Animated" } else { "Static" }
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
                        position_type: PositionType::Absolute,
                        left: px(PANEL_WIDTH),
                        right: px(0),
                        top: px(0),
                        bottom: px(0),
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
                            PresentationWidth(width),
                            Node {
                                width: px(width),
                                max_width: percent(90),
                                ..default()
                            },
                        ));
                    }
                });
            demo.status.notice = "Hover and press the exported Flash button.".into();
        }
        Target::None => return,
    }
    demo.status.ready = true;
}

fn fit(
    windows: Query<&Window>,
    demo: Res<Demo>,
    mut actors: Query<(&Framing, &mut Transform), With<Actor>>,
    graphics: Res<Assets<VabGraphic>>,
    mut images: Query<(&PresentationWidth, &VabImageNode, &mut Node)>,
) {
    let Ok(window) = windows.single() else { return };
    for (frame, mut transform) in &mut actors {
        let scale = ((window.width() - PANEL_WIDTH).max(1.0) * 0.88 / frame.size.x)
            .min(window.height() * 0.82 / frame.size.y)
            .clamp(0.01, 8.0)
            * demo.status.zoom;
        // Zoom around the fixed framing center, never the current frame bounds.
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
        .min((window.width() - PANEL_WIDTH).max(1.0) * 0.90 / widest)
        .clamp(0.05, 1.0);
    for (width, _, mut node) in &mut images {
        let desired = px(width.0 * factor * demo.status.zoom);
        if node.width != desired {
            node.width = desired;
        }
    }
}

fn refresh_status(
    mut demo: ResMut<Demo>,
    assets: Res<Assets<VabAsset>>,
    actor: Query<(&VabAssetHandle, &VabPlayer, &VabSkin), With<Actor>>,
    ui: Query<&VabUiPlayback>,
) {
    if let Ok((handle, player, skin)) = actor.single()
        && let Some(asset) = assets.get(&handle.0)
    {
        if let Some(clip) = asset.baked.clips.get(player.clip()) {
            demo.status.clip = clip.name.clone();
        }
        demo.status.playing = player.playing;
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
        demo.status.playing = player.playing;
    }
}

#[cfg(test)]
mod tests;
