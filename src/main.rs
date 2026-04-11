//! Starship lander — Bevy 2D (native + WASM).

mod camera;
mod collision;
mod constants;
mod game_flow;
mod physics;
mod planets;
mod ship;
mod terrain;
mod ui;

use bevy::prelude::*;

fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Starship Lander".into(),
                        resolution: (1280, 720).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(bevy::log::LogPlugin {
                    level: bevy::log::Level::INFO,
                    ..default()
                }),
        )
        .add_plugins((
            camera::CameraPlugin,
            game_flow::GameFlowPlugin,
            ui::UiPlugin,
        ))
        .run();
}
