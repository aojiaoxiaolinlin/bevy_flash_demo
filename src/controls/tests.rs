use super::*;

#[test]
fn native_panel_rebuild_and_activation_work_without_a_browser() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<Demo>()
        .add_message::<Action>()
        .add_observer(activate)
        .add_systems(Startup, setup)
        .add_systems(Update, (panel, status_text).chain());
    app.update();
    let camera_count = app
        .world_mut()
        .query_filtered::<Entity, With<Camera2d>>()
        .iter(app.world())
        .count();
    assert_eq!(camera_count, 2);
    let entity = app
        .world_mut()
        .query::<(Entity, &Action)>()
        .iter(app.world())
        .find(|(_, action)| matches!(action, Action::SelectPage { page: Page::Skins }))
        .unwrap()
        .0;
    app.world_mut().trigger(Activate { entity });
    let actions: Vec<_> = app
        .world_mut()
        .resource_mut::<Messages<Action>>()
        .drain()
        .collect();
    assert!(matches!(
        actions.as_slice(),
        [Action::SelectPage { page: Page::Skins }]
    ));
    {
        let mut demo = app.world_mut().resource_mut::<Demo>();
        demo.status.ready = true;
        demo.status.clips = vec!["idle".into(), "attack".into()];
        demo.status.skins = vec![(
            "body".into(),
            vec!["red".into(), "blue".into()],
            "red".into(),
        )];
    }
    app.update();
    assert!(!app.world().entities().contains(entity));
    assert!(
        app.world_mut()
            .query::<&Action>()
            .iter(app.world())
            .any(|a| matches!(a, Action::Play { name } if name == "attack"))
    );
    // A second update exercises the retained panel and disjoint Text queries.
    app.update();
}
#[test]
#[ignore = "requires a WebGPU-capable GPU"]
fn linear_ui_camera_preserves_the_srgb_world_output() {
    use bevy::{
        camera::RenderTarget,
        render::{
            gpu_readback::{Readback, ReadbackComplete},
            pipelined_rendering::PipelinedRenderingPlugin,
            render_resource::{TextureFormat, TextureUsages},
            renderer::RenderDevice,
        },
        window::ExitCondition,
        winit::WinitPlugin,
    };
    use std::{
        sync::{Arc, Mutex},
        time::{Duration, Instant},
    };
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: None,
                exit_condition: ExitCondition::DontExit,
                ..default()
            })
            .disable::<WinitPlugin>()
            .disable::<PipelinedRenderingPlugin>(),
    )
    .init_resource::<Demo>()
    .add_message::<Action>()
    .add_systems(Startup, setup)
    .add_systems(Update, (panel, status_text).chain());
    while app.plugins_state() != bevy::app::PluginsState::Ready {
        app.update();
    }
    app.finish();
    app.cleanup();
    {
        let mut demo = app.world_mut().resource_mut::<Demo>();
        demo.status.ready = true;
        demo.status.clips = [
            "STF", "OTF", "WAI", "ATT", "UDA", "BTD", "MIS", "MGS", "MGF", "MGE", "DEA", "OWK",
            "PER",
        ]
        .into_iter()
        .map(String::from)
        .collect();
        demo.status.clip = "ATT".into();
        demo.status.fallback = "STF".into();
        demo.status.playing = true;
        demo.status.zoom = 1.0;
    }
    let mut image = Image::new_target_texture(800, 600, TextureFormat::Rgba8UnormSrgb, None);
    image.texture_descriptor.usage |= TextureUsages::COPY_SRC;
    let target = app.world_mut().resource_mut::<Assets<Image>>().add(image);
    app.update();
    let cameras: Vec<_> = app
        .world_mut()
        .query_filtered::<Entity, With<Camera2d>>()
        .iter(app.world())
        .collect();
    for camera in cameras {
        app.world_mut()
            .entity_mut(camera)
            .insert(RenderTarget::Image(target.clone().into()));
    }
    app.world_mut().spawn((
        Sprite::from_color(Color::srgb(1.0, 0.0, 0.0), Vec2::splat(100.0)),
        Transform::from_xyz(150.0, 0.0, 0.0),
    ));
    let warmup = Instant::now();
    while warmup.elapsed() < Duration::from_millis(500) {
        app.update();
        std::thread::sleep(Duration::from_millis(5));
    }
    let pixels = Arc::new(Mutex::new(None));
    let result = pixels.clone();
    app.world_mut()
        .spawn(Readback::texture(target.clone()))
        .observe(move |event: On<ReadbackComplete>| {
            *result.lock().unwrap() = Some(event.data.clone());
        });
    let start = Instant::now();
    let bytes = loop {
        app.update();
        if let Some(data) = pixels.lock().unwrap().take() {
            break data;
        }
        assert!(
            start.elapsed() < Duration::from_secs(15),
            "camera readback timeout"
        );
        std::thread::sleep(Duration::from_millis(5));
    };
    let stride = RenderDevice::align_copy_bytes_per_row(800 * 4);
    let mut snapshot = app
        .world()
        .resource::<Assets<Image>>()
        .get(&target)
        .unwrap()
        .clone();
    snapshot.data = Some(
        bytes
            .chunks(stride)
            .take(600)
            .flat_map(|row| row[..800 * 4].iter().copied())
            .collect(),
    );
    snapshot
        .try_into_dynamic()
        .unwrap()
        .save(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/control-panel.png"))
        .unwrap();
    let red = &bytes[300 * stride + 550 * 4..][..4];
    assert!(
        red[0] > 180 && red[1] < 60 && red[2] < 60,
        "UI camera overwrote world: {red:?}"
    );
    let panel = &bytes[300 * stride + 20 * 4..][..4];
    assert!(
        panel[2] > panel[0] && panel[3] > 240,
        "panel missing: {panel:?}"
    );
}
