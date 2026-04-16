//! Player ship: velocity, fuel, foot tracking for landing detection.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use crate::constants::{
    FOOT_SEP_X, INITIAL_FUEL, SHIP_FOOT_OFFSET_Y, SHIP_HULL_BOTTOM_OFFSET_Y, ship_spawn_y,
};

#[inline]
fn hull_base_y() -> f32 {
    SHIP_HULL_BOTTOM_OFFSET_Y
}

#[derive(Component)]
pub struct Ship {
    pub velocity: Vec2,
    pub fuel: f32,
    /// Foot line y before this frame's integration (for ground crossing).
    pub foot_prev: f32,
    /// Hull bottom edge y before this frame's integration (hits terrain before feet).
    pub hull_bottom_prev: f32,
}

#[derive(Component)]
pub struct ShipRoot;

/// Local-space offsets for exhaust (main, left side, right side) relative to ship center.
pub fn thruster_local_offsets() -> (Vec3, Vec3, Vec3) {
    (
        Vec3::new(0.0, -22.0, 0.0),
        Vec3::new(-FOOT_SEP_X, -19.0, 0.0),
        Vec3::new(FOOT_SEP_X, -19.0, 0.0),
    )
}

fn hull_triangle_mesh() -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            [0.0, 24.0, 0.0],
            [-19.0, hull_base_y(), 0.0],
            [19.0, hull_base_y(), 0.0],
        ],
    );
    mesh.insert_indices(Indices::U32(vec![0, 1, 2]));
    mesh
}

pub fn spawn_ship(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) -> Entity {
    let hull = meshes.add(hull_triangle_mesh());
    let leg_mesh = meshes.add(Rectangle::new(5.5, 10.0));
    let engine_mesh = meshes.add(Rectangle::new(5.0, 4.0));

    let hull_mat = materials.add(Color::srgb(0.72, 0.74, 0.82));
    let leg_mat = materials.add(Color::srgb(0.55, 0.58, 0.65));
    let engine_mat = materials.add(Color::srgb(0.45, 0.48, 0.55));

    let y = ship_spawn_y();
    commands
        .spawn((
            ShipRoot,
            Ship {
                velocity: Vec2::ZERO,
                fuel: INITIAL_FUEL,
                foot_prev: y + SHIP_FOOT_OFFSET_Y,
                hull_bottom_prev: y + SHIP_HULL_BOTTOM_OFFSET_Y,
            },
            Transform::from_xyz(0.0, y, 1.0),
        ))
        .with_children(|parent| {
            parent.spawn((
                Mesh2d(hull),
                MeshMaterial2d(hull_mat),
                Transform::from_xyz(0.0, 0.0, 0.0),
            ));
            parent.spawn((
                Mesh2d(leg_mesh.clone()),
                MeshMaterial2d(leg_mat.clone()),
                Transform::from_xyz(-19.0, -21.0, 0.1),
            ));
            parent.spawn((
                Mesh2d(leg_mesh),
                MeshMaterial2d(leg_mat),
                Transform::from_xyz(19.0, -21.0, 0.1),
            ));
            parent.spawn((
                Mesh2d(engine_mesh),
                MeshMaterial2d(engine_mat),
                Transform::from_xyz(0.0, -20.0, 0.2),
            ));
        })
        .id()
}

#[inline]
pub fn ship_foot_y(transform: &Transform) -> f32 {
    transform.translation.y + SHIP_FOOT_OFFSET_Y
}

#[inline]
pub fn ship_hull_bottom_y(transform: &Transform) -> f32 {
    transform.translation.y + SHIP_HULL_BOTTOM_OFFSET_Y
}

#[inline]
pub fn ship_left_foot_x(transform: &Transform) -> f32 {
    transform.translation.x - FOOT_SEP_X
}

#[inline]
pub fn ship_right_foot_x(transform: &Transform) -> f32 {
    transform.translation.x + FOOT_SEP_X
}
