mod bounds;
mod bridge;
mod catalog;
mod demos;

use bevy::prelude::*;
use bevy_flash::{FlashPlayerPlugin, vab_ui::VabUiPlugin};

fn main() {
    let mut window = Window {
        title: "bevy_flash · WebGPU showcase".into(),
        resolution: (1080, 760).into(),
        ..default()
    };
    #[cfg(target_arch = "wasm32")]
    {
        window.canvas = Some("#bevy".into());
        window.fit_canvas_to_parent = true;
        window.prevent_default_event_handling = true;
    }
    // Native validation uses the same renderer, with keyboard page navigation.
    #[cfg(not(target_arch = "wasm32"))]
    {
        window.resizable = true;
    }
    App::new()
        .insert_resource(ClearColor(Color::srgb_u8(102, 102, 102)))
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(window),
                ..default()
            }),
            FlashPlayerPlugin,
            VabUiPlugin,
            demos::DemoPlugin,
        ))
        .run();
}
