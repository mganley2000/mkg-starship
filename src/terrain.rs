//! Procedural terrain with flat landing pads.

use std::f32::consts::PI;

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::prelude::*;
use rand::Rng;
use rand::rngs::StdRng;

use crate::constants::{TERRAIN_FLOOR_Y, TERRAIN_SAMPLES, WORLD_WIDTH};

/// Top surface polyline, left → right (x increasing).
#[derive(Resource, Clone)]
pub struct Terrain {
    pub points: Vec<Vec2>,
    pub pads: Vec<LandingPad>,
}

#[derive(Clone, Copy, Debug)]
pub struct LandingPad {
    pub x_min: f32,
    pub x_max: f32,
    pub y_top: f32,
}

#[derive(Component)]
pub struct TerrainRoot;

/// Height of terrain top at world x (linear between samples).
pub fn terrain_height_at(terrain: &Terrain, x: f32) -> f32 {
    let pts = &terrain.points;
    if pts.is_empty() {
        return TERRAIN_FLOOR_Y;
    }
    if x <= pts[0].x {
        return pts[0].y;
    }
    let last = pts[pts.len() - 1];
    if x >= last.x {
        return last.y;
    }
    for w in pts.windows(2) {
        let a = w[0];
        let b = w[1];
        if x >= a.x && x <= b.x {
            let t = (x - a.x) / (b.x - a.x).max(1e-6);
            return a.y + t * (b.y - a.y);
        }
    }
    last.y
}

pub fn is_on_pad(terrain: &Terrain, x: f32, foot_y: f32, tolerance: f32) -> Option<&LandingPad> {
    for pad in &terrain.pads {
        if x >= pad.x_min && x <= pad.x_max && (foot_y - pad.y_top).abs() <= tolerance {
            return Some(pad);
        }
    }
    None
}

/// Random terrain with `pad_count` flat regions (2–3).
pub fn generate_terrain(rng: &mut StdRng, pad_count: usize) -> Terrain {
    let half = WORLD_WIDTH * 0.5;
    let x_min = -half;
    let x_max = half;
    let n = TERRAIN_SAMPLES;
    let step = (x_max - x_min) / (n - 1) as f32;

    let mut ys: Vec<f32> = Vec::with_capacity(n);
    let base = -120.0_f32;
    for i in 0..n {
        let t = i as f32 / (n - 1) as f32;
        let wobble = (t * PI * 6.0).sin() * 55.0 + (t * PI * 13.0).sin() * 28.0;
        let noise = rng.gen_range(-18.0..18.0);
        let y = base + wobble + noise * 0.4 + (i as f32 * 0.35);
        ys.push(y);
    }

    let pad_count = pad_count.clamp(2, 3);
    let mut pads = Vec::new();
    let margin = WORLD_WIDTH * 0.12;
    for _ in 0..pad_count {
        let px0 = rng.gen_range((x_min + margin)..(x_max - margin - 140.0));
        let width = rng.gen_range(90.0..140.0);
        let px1 = (px0 + width).min(x_max - margin);
        let i0 = (((px0 - x_min) / step).round() as usize).min(n - 2);
        let i1 = (((px1 - x_min) / step).round() as usize)
            .max(i0 + 2)
            .min(n - 1);
        let flat_y = rng.gen_range(-95.0..35.0);
        for idx in i0..=i1 {
            ys[idx] = flat_y;
        }
        pads.push(LandingPad {
            x_min: x_min + i0 as f32 * step,
            x_max: x_min + i1 as f32 * step,
            y_top: flat_y,
        });
    }

    let mut points: Vec<Vec2> = (0..n)
        .map(|i| Vec2::new(x_min + i as f32 * step, ys[i]))
        .collect();

    for _ in 0..2 {
        let copy = points.clone();
        for i in 1..points.len() - 1 {
            let on_pad = pads
                .iter()
                .any(|p| points[i].x >= p.x_min && points[i].x <= p.x_max);
            if !on_pad {
                points[i].y = copy[i - 1].y * 0.25 + copy[i].y * 0.5 + copy[i + 1].y * 0.25;
            }
        }
    }

    Terrain { points, pads }
}

pub fn build_terrain_mesh(terrain: &Terrain) -> Mesh {
    let pts = &terrain.points;
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for w in pts.windows(2) {
        let p0 = w[0];
        let p1 = w[1];
        let base = positions.len() as u32;
        positions.push([p0.x, TERRAIN_FLOOR_Y, 0.0]);
        positions.push([p1.x, TERRAIN_FLOOR_Y, 0.0]);
        positions.push([p1.x, p1.y, 0.0]);
        positions.push([p0.x, p0.y, 0.0]);
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float32x3(positions),
    );
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

pub fn spawn_terrain_entity(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    terrain: &Terrain,
) -> Entity {
    let mesh = build_terrain_mesh(terrain);
    commands
        .spawn((
            TerrainRoot,
            Mesh2d(meshes.add(mesh)),
            MeshMaterial2d(materials.add(Color::srgb(0.2, 0.55, 0.32))),
            Transform::default(),
        ))
        .id()
}
