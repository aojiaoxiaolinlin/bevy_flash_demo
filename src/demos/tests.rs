use super::*;

#[test]
#[ignore = "initializes the GPU renderer and loads the real demo asset"]
fn configured_fallback_preserves_current_action_and_runs_after_completion() {
    use bevy::{
        render::pipelined_rendering::PipelinedRenderingPlugin, window::ExitCondition,
        winit::WinitPlugin,
    };
    use std::time::{Duration, Instant};
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: None,
                exit_condition: ExitCondition::DontExit,
                ..default()
            })
            .set(AssetPlugin {
                file_path: format!("{}/assets", env!("CARGO_MANIFEST_DIR")),
                meta_check: bevy::asset::AssetMetaCheck::Never,
                ..default()
            })
            .disable::<WinitPlugin>()
            .disable::<PipelinedRenderingPlugin>(),
    )
    .add_plugins((
        bevy_flash::FlashPlayerPlugin,
        bevy_flash::vab_ui::VabUiPlugin,
        DemoPlugin,
    ));
    while app.plugins_state() != bevy::app::PluginsState::Ready {
        app.update();
    }
    app.finish();
    app.cleanup();
    let start = Instant::now();
    while !app.world().resource::<Demo>().status.ready {
        app.update();
        assert!(app.world().resource::<Demo>().status.error.is_none());
        assert!(
            start.elapsed() < Duration::from_secs(30),
            "asset loading timeout"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
    let names = app.world().resource::<Demo>().status.clips.clone();
    let actor = app
        .world_mut()
        .query_filtered::<Entity, With<Actor>>()
        .single(app.world())
        .unwrap();
    let before = app.world().get::<VabPlayer>(actor).unwrap().clip();
    app.world_mut()
        .resource_mut::<Messages<Action>>()
        .write(Action::Fallback {
            name: names[1].clone(),
        });
    app.update();
    assert_eq!(app.world().get::<VabPlayer>(actor).unwrap().clip(), before);
    assert_eq!(app.world().resource::<Demo>().status.fallback, names[1]);
    app.world_mut()
        .resource_mut::<Messages<Action>>()
        .write(Action::Play {
            name: names[2].clone(),
        });
    app.update();
    app.world_mut()
        .resource_scope(|world, assets: Mut<Assets<VabAsset>>| {
            let handle = world.get::<VabAssetHandle>(actor).unwrap().0.clone();
            let asset = assets.get(&handle).unwrap();
            let mut player = world.get_mut::<VabPlayer>(actor).unwrap();
            let seconds = asset.baked.clips[player.clip()].frames.len() as f64
                / f64::from(asset.playback_frame_rate())
                + 0.01;
            player.advance(asset, seconds);
            assert_eq!(asset.baked.clips[player.clip()].name, names[1]);
            assert!(player.is_looping());
        });
}
