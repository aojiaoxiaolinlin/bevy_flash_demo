use bevy::prelude::{Camera2d, Local, MouseButton, PluginGroup, Touches, Visibility};
use bevy::window::{Window, WindowPlugin};
use bevy::{
    app::{App, Startup, Update},
    asset::{AssetServer, Assets},
    input::ButtonInput,
    math::Vec3,
    prelude::{Commands, KeyCode, Query, Res, ResMut, Transform},
    DefaultPlugins,
};
use bevy_flash::bundle::FlashAnimation;
use bevy_flash::{assets::SwfMovie, plugin::FlashPlugin};
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
    commands.spawn(Camera2d::default());
    commands.spawn((
        FlashAnimation {
            name: Some(String::from("_mc")),
            swf_movie: assert_server.load("spirit2724src.swf"),
            ..Default::default()
        },
        Transform::from_translation(Vec3::new(-500.0, 0.0, 0.0)).with_scale(Vec3::splat(1.0)),
        Visibility::default(),
    ));
    commands.spawn((
        FlashAnimation {
            name: Some(String::from("m")),
            swf_movie: assert_server.load("131381-idle.swf"),
            ..Default::default()
        },
        Transform::from_scale(Vec3::splat(4.0)).with_translation(Vec3::new(-500.0, 0.0, 0.0)),
        Visibility::default(),
    ));
    commands.spawn((
        FlashAnimation {
            name: Some(String::from("c")),
            swf_movie: assert_server.load("frames.swf"),
            ..Default::default()
        },
        Transform::from_scale(Vec3::splat(1.0)).with_translation(Vec3::new(-1300.0, 1100.0, 0.0)),
        Visibility::default(),
    ));
}

fn control(
    mut query: Query<&mut FlashAnimation>,
    mut swf_movies: ResMut<Assets<SwfMovie>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    mut current_frame: Local<CurrentFrame>,
) {
    let mut control = |query: &mut Query<'_, '_, &mut FlashAnimation>, frame: Option<u16>| {
        query.iter_mut().for_each(|flash_animation| {
            if let Some(swf_movie) = swf_movies.get_mut(flash_animation.swf_movie.id()) {
                if flash_animation.name == Some(String::from("_mc")) {
                    if let Some(frame) = frame {
                        current_frame.0 = frame;
                        swf_movie.root_movie_clip.goto_frame(
                            &mut swf_movie.movie_library,
                            frame,
                            true,
                        );
                    } else {
                        if current_frame.0 >= 110 {
                            current_frame.0 = 0;
                        } else {
                            current_frame.0 += 10;
                        }

                        swf_movie.root_movie_clip.goto_frame(
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
