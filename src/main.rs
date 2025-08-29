//#![windows_subsystem = "windows"]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

use bevy::prelude::*;

mod engine;
mod ui;

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
        .add_plugins(ui::MyUiPlugin)
        .run();
}
