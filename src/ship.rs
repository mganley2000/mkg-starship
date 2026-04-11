//! Player ship: velocity, fuel, foot tracking for landing detection.

use bevy::prelude::*;

use crate::constants::{INITIAL_FUEL, SHIP_HALF_HEIGHT, SHIP_HALF_WIDTH, ship_spawn_y};

#[derive(Component)]
pub struct Ship {
    pub velocity: Vec2,
    pub fuel: f32,
    /// Foot y before this frame's integration (for ground crossing).
    pub foot_prev: f32,
}

#[derive(Component)]
pub struct ShipRoot;

pub fn spawn_ship(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) -> Entity {
    let mesh = meshes.add(Rectangle::new(
        SHIP_HALF_WIDTH * 2.0,
        SHIP_HALF_HEIGHT * 2.0,
    ));
    let y = ship_spawn_y();
    commands
        .spawn((
            ShipRoot,
            Ship {
                velocity: Vec2::ZERO,
                fuel: INITIAL_FUEL,
                foot_prev: y - SHIP_HALF_HEIGHT,
            },
            Mesh2d(mesh),
            MeshMaterial2d(materials.add(Color::srgb(0.78, 0.8, 0.88))),
            Transform::from_xyz(0.0, y, 1.0),
        ))
        .id()
}

pub fn ship_foot_x(transform: &Transform) -> f32 {
    transform.translation.x
}

pub fn ship_foot_y(transform: &Transform) -> f32 {
    transform.translation.y - SHIP_HALF_HEIGHT
}
