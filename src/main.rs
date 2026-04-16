//! Starship lander — Bevy 2D (native + WASM).

mod ambient_vfx;
mod camera;
mod collision;
mod crash_explosion;
mod constants;
mod exhaust;
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
            ambient_vfx::AmbientVfxPlugin,
            game_flow::GameFlowPlugin,
            crash_explosion::CrashExplosionPlugin,
            exhaust::ExhaustPlugin,
            ui::UiPlugin,
        ))
        .run();
}
