use bevy::prelude::*;

mod engine;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(
            WindowPlugin {
                primary_window: Some(Window {
                    title: "Bevy Tetris".to_string(),
                    ..Default::default()
                }),
                ..Default::default()
            }
        ))
        .add_plugins(engine::scene::ScenePlugin)
        .run();
}
