//! 2D camera and window setup.

use bevy::camera::ScalingMode;
use bevy::prelude::*;
use bevy::window::WindowResized;

use crate::constants::{WORLD_HEIGHT, WORLD_WIDTH};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(Update, on_window_resize);
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::Fixed {
                width: WORLD_WIDTH,
                height: WORLD_HEIGHT,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}

fn on_window_resize(
    mut resize: MessageReader<WindowResized>,
    mut projections: Query<&mut Projection, With<Camera2d>>,
) {
    for _ in resize.read() {
        for mut proj in &mut projections {
            if let Projection::Orthographic(o) = proj.as_mut() {
                o.scaling_mode = ScalingMode::Fixed {
                    width: WORLD_WIDTH,
                    height: WORLD_HEIGHT,
                };
            }
        }
    }
}
