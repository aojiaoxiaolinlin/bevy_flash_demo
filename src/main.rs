use bevy::prelude::{Local, MouseButton, PluginGroup, Touches};
use bevy::window::{Window, WindowPlugin};
use bevy::{
    app::{App, Startup, Update},
    asset::{AssetServer, Assets, Handle},
    input::ButtonInput,
    math::Vec3,
    prelude::{Camera2dBundle, Commands, KeyCode, Query, Res, ResMut, SpatialBundle, Transform},
    DefaultPlugins,
};
use bevy_flash::{
    assets::SwfMovie,
    bundle::{Swf, SwfBundle},
    plugin::FlashPlugin,
};
#[derive(Default)]
struct CurrentFrame(u16);

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: (1024., 768.).into(),
                    // provide the ID selector string here
                    canvas: Some("#bevy".into()),
                    // ... any other window properties ...
                    ..Default::default()
                }),
                ..Default::default()
            }),
            FlashPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, control)
        .run();
}

fn setup(mut commands: Commands, assert_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(SwfBundle {
        swf_handle: assert_server.load("spirit2724src.swf"),
        swf: Swf {
            name: Some(String::from("_mc")),
            ..Default::default()
        },
        spatial: SpatialBundle {
            transform: Transform::from_translation(Vec3::new(-500.0, 0.0, 0.0))
                .with_scale(Vec3::splat(1.0)),
            ..Default::default()
        },
        ..Default::default()
    });
    commands.spawn(SwfBundle {
        swf_handle: assert_server.load("131381-idle.swf"),
        spatial: SpatialBundle {
            transform: Transform::from_scale(Vec3::splat(4.0))
                .with_translation(Vec3::new(-500.0, 0.0, 0.0)),
            ..Default::default()
        },
        ..Default::default()
    });
}

fn control(
    mut query: Query<(&mut Swf, &Handle<SwfMovie>)>,
    mut swf_movies: ResMut<Assets<SwfMovie>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    mut current_frame: Local<CurrentFrame>,
) {
    let mut control = |query: &mut Query<'_, '_, (&mut Swf, &Handle<SwfMovie>)>,
                       frame: Option<u16>| {
        query.iter_mut().for_each(|(mut swf, handle_swf_movie)| {
            if let Some(swf_movie) = swf_movies.get_mut(handle_swf_movie.id()) {
                if swf.is_target_movie_clip() {
                    if let Some(frame) = frame {
                        current_frame.0 = frame;
                        swf.root_movie_clip
                            .goto_frame(&mut swf_movie.movie_library, frame, true);
                    } else {
                        if current_frame.0 >= 110 {
                            current_frame.0 = 0;
                        } else {
                            current_frame.0 += 10;
                        }

                        swf.root_movie_clip.goto_frame(
                            &mut swf_movie.movie_library,
                            current_frame.0,
                            true,
                        );
                    }
                }
            }
        });
    };

    if keyboard_input.just_released(KeyCode::KeyW) {
        control(&mut query, Some(0));
    }

    if keyboard_input.just_released(KeyCode::KeyA) {
        control(&mut query, Some(10));
    }

    if keyboard_input.just_released(KeyCode::KeyS) {
        control(&mut query, Some(20));
    }

    if keyboard_input.just_released(KeyCode::KeyD) {
        control(&mut query, Some(30));
    }

    if keyboard_input.just_released(KeyCode::KeyF) {
        control(&mut query, Some(40));
    }

    if keyboard_input.just_released(KeyCode::KeyG) {
        control(&mut query, Some(50));
    }

    if keyboard_input.just_released(KeyCode::KeyH) {
        control(&mut query, Some(60));
    }

    if keyboard_input.just_released(KeyCode::KeyJ) {
        control(&mut query, Some(70));
    }

    if keyboard_input.just_released(KeyCode::KeyK) {
        control(&mut query, Some(80));
    }

    if keyboard_input.just_released(KeyCode::KeyL) {
        control(&mut query, Some(90));
    }

    if keyboard_input.just_released(KeyCode::KeyM) {
        control(&mut query, Some(100));
    }

    if keyboard_input.just_released(KeyCode::KeyN) {
        control(&mut query, Some(110));
    }

    if mouse_button_input.just_pressed(MouseButton::Left) {
        control(&mut query, None);
    }

    for _touch in touches.iter_just_pressed() {
        control(&mut query, None);
    }
}
