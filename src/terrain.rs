//! Procedural terrain with flat landing pads.

use std::f32::consts::PI;

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::prelude::*;
use rand::Rng;
use rand::rngs::StdRng;

use crate::constants::{TERRAIN_FLOOR_Y, TERRAIN_SAMPLES, WORLD_WIDTH};
use crate::planets::CelestialBody;

/// Tunable terrain shape for a body (heightfield before pads).
#[derive(Clone, Copy, Debug)]
pub struct TerrainProfile {
    pub base: f32,
    pub sin_amp: [f32; 4],
    pub noise_half: f32,
    pub micro_spike_prob: f32,
    pub micro_spike_half: f32,
    pub pad_y_min: f32,
    pub pad_y_max: f32,
    pub smoothing_passes: u32,
}

impl TerrainProfile {
    fn rocky_default() -> Self {
        Self {
            base: -305.0,
            sin_amp: [58.0, 40.0, 24.0, 12.0],
            noise_half: 36.0,
            micro_spike_prob: 0.055,
            micro_spike_half: 22.0,
            pad_y_min: -335.0,
            pad_y_max: -245.0,
            smoothing_passes: 2,
        }
    }

    fn gas_giant_smooth() -> Self {
        Self {
            base: -305.0,
            sin_amp: [34.0, 22.0, 14.0, 7.5],
            noise_half: 18.0,
            micro_spike_prob: 0.03,
            micro_spike_half: 11.0,
            pad_y_min: -330.0,
            pad_y_max: -250.0,
            smoothing_passes: 3,
        }
    }
}

/// Per-body terrain: gas giants smoother; icy moons tweaked; Io volcanic.
pub fn terrain_profile_for(body: CelestialBody) -> TerrainProfile {
    match body {
        CelestialBody::Saturn | CelestialBody::Uranus => TerrainProfile::gas_giant_smooth(),
        CelestialBody::Enceladus | CelestialBody::Europa => {
            let mut p = TerrainProfile::gas_giant_smooth();
            p.base -= 6.0;
            p.noise_half *= 0.85;
            p
        }
        CelestialBody::Io => {
            let mut p = TerrainProfile::rocky_default();
            p.sin_amp[0] += 14.0;
            p.micro_spike_prob += 0.045;
            p.micro_spike_half += 8.0;
            p.noise_half += 6.0;
            p.smoothing_passes = p.smoothing_passes.max(2);
            p
        }
        CelestialBody::Ganymede => {
            let mut p = TerrainProfile::rocky_default();
            p.base -= 6.0;
            p
        }
        CelestialBody::Callisto => {
            let mut p = TerrainProfile::rocky_default();
            p.base -= 10.0;
            p.noise_half += 12.0;
            p.micro_spike_prob += 0.04;
            p
        }
        CelestialBody::Titan => {
            let mut p = TerrainProfile::rocky_default();
            p.base += 2.0;
            p.sin_amp[1] += 8.0;
            p.smoothing_passes = 1;
            p
        }
        CelestialBody::Mercury => {
            let mut p = TerrainProfile::rocky_default();
            p.base -= 8.0;
            p
        }
        CelestialBody::Venus => {
            let mut p = TerrainProfile::rocky_default();
            p.base += 4.0;
            p.sin_amp[0] += 6.0;
            p
        }
        CelestialBody::Earth => TerrainProfile::rocky_default(),
        CelestialBody::Mars => {
            let mut p = TerrainProfile::rocky_default();
            p.base -= 12.0;
            p.noise_half += 6.0;
            p
        }
        CelestialBody::Jupiter => {
            let mut p = TerrainProfile::rocky_default();
            p.sin_amp[1] += 10.0;
            p.micro_spike_prob += 0.02;
            p
        }
        CelestialBody::Neptune => {
            let mut p = TerrainProfile::rocky_default();
            p.base -= 4.0;
            p.sin_amp[3] += 4.0;
            p
        }
        CelestialBody::Pluto => {
            let mut p = TerrainProfile::rocky_default();
            p.base -= 18.0;
            p.noise_half += 8.0;
            p.micro_spike_prob += 0.04;
            p
        }
    }
}

/// Top surface polyline, left → right (x increasing).
#[derive(Resource, Clone)]
pub struct Terrain {
    pub points: Vec<Vec2>,
    pub pads: Vec<LandingPad>,
    /// Io: one volcanic vent in a recessed surface zone.
    pub io_volcano: Option<VolcanoVent>,
    /// Mercury: 1–2 volcanic vents (cone + particles).
    pub mercury_volcanoes: Vec<VolcanoVent>,
    /// Enceladus: 2–3 cryo-plume sites (no volcano mesh).
    pub enceladus_plumes: Vec<Vec2>,
}

#[derive(Clone, Copy, Debug)]
pub struct LandingPad {
    pub x_min: f32,
    pub x_max: f32,
    pub y_top: f32,
}

/// Volcanic vent placement and mesh scale (see `generate_terrain`).
#[derive(Clone, Copy, Debug)]
pub struct VolcanoVent {
    pub pos: Vec2,
    /// Vertical scale of the mound mesh (1.0 default; 50% of vents use 2× or 3×).
    pub height_scale: f32,
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

/// Highest terrain under a horizontal span (hull bottom may hit a peak before feet).
pub fn terrain_max_height_in_span(terrain: &Terrain, x_center: f32, half_width: f32) -> f32 {
    const N: usize = 9;
    let mut m = f32::NEG_INFINITY;
    for i in 0..N {
        let t = i as f32 / (N - 1).max(1) as f32;
        let x = x_center - half_width + t * (2.0 * half_width);
        m = m.max(terrain_height_at(terrain, x));
    }
    m
}

fn pick_x_avoiding_pads(
    rng: &mut StdRng,
    x_min: f32,
    x_max: f32,
    pads: &[LandingPad],
    margin: f32,
) -> f32 {
    for _ in 0..100 {
        let x = rng.gen_range(x_min..x_max);
        let on_pad = pads.iter().any(|p| {
            x >= p.x_min - margin && x <= p.x_max + margin
        });
        if !on_pad {
            return x;
        }
    }
    rng.gen_range(x_min..x_max)
}

/// How far below a typical pad height the volcano plateau sits (recessed vent).
const VOLCANO_SURFACE_RECESS: f32 = 20.0;

fn random_volcano_height_scale(rng: &mut StdRng) -> f32 {
    if rng.gen_bool(0.5) {
        if rng.gen_bool(0.5) {
            2.0
        } else {
            3.0
        }
    } else {
        1.0
    }
}

fn blend_volcano_edges(ys: &mut [f32], ys_orig: &[f32], i0: usize, i1: usize, plateau: f32) {
    const BLEND: usize = 6;
    for k in 1..=BLEND {
        let t = k as f32 / (BLEND as f32 + 1.0);
        let li = i0.saturating_sub(k);
        if li < i0 {
            ys[li] = ys_orig[li] * (1.0 - t) + plateau * t;
        }
        let ri = i1 + k;
        if ri < ys.len() && ri > i1 {
            ys[ri] = ys_orig[ri] * (1.0 - t) + plateau * t;
        }
    }
}

/// Flatten a span of the heightfield for a volcano vent (avoids landing pads and other vents).
/// Lowers the plateau by [`VOLCANO_SURFACE_RECESS`] so the vent sits in a depression.
/// Returns the zone bounds (at the recessed height) and a vertical mesh scale for the cone.
fn try_flatten_volcano_zone(
    rng: &mut StdRng,
    x_min: f32,
    x_max: f32,
    step: f32,
    n: usize,
    landing_pads: &[LandingPad],
    existing_vents: &[LandingPad],
    profile: &TerrainProfile,
    ys: &mut [f32],
) -> Option<(LandingPad, f32)> {
    const WIDTH_MIN: f32 = 46.0;
    const WIDTH_MAX: f32 = 74.0;
    const PAD_MARGIN: f32 = 52.0;
    const VENT_MARGIN: f32 = 44.0;
    let edge = WORLD_WIDTH * 0.08;

    for _ in 0..160 {
        let width = rng.gen_range(WIDTH_MIN..WIDTH_MAX);
        let px0 = rng.gen_range((x_min + edge)..(x_max - edge - width).max(x_min + edge + 20.0));
        let px1 = px0 + width;

        let overlaps_pad = landing_pads.iter().any(|p| {
            !(px1 + PAD_MARGIN < p.x_min || px0 - PAD_MARGIN > p.x_max)
        });
        if overlaps_pad {
            continue;
        }

        let overlaps_vent = existing_vents.iter().any(|p| {
            !(px1 + VENT_MARGIN < p.x_min || px0 - VENT_MARGIN > p.x_max)
        });
        if overlaps_vent {
            continue;
        }

        let i0 = (((px0 - x_min) / step).round() as usize).min(n.saturating_sub(2));
        let i1 = (((px1 - x_min) / step).round() as usize)
            .max(i0 + 2)
            .min(n - 1);
        let flat_y = rng.gen_range(profile.pad_y_min..profile.pad_y_max);
        let plateau = flat_y - VOLCANO_SURFACE_RECESS;

        let ys_orig = ys.to_vec();
        for idx in i0..=i1 {
            ys[idx] = plateau;
        }
        blend_volcano_edges(ys, &ys_orig, i0, i1, plateau);

        let height_scale = random_volcano_height_scale(rng);

        return Some((
            LandingPad {
                x_min: x_min + i0 as f32 * step,
                x_max: x_min + i1 as f32 * step,
                y_top: plateau,
            },
            height_scale,
        ));
    }
    None
}

pub fn is_on_pad(terrain: &Terrain, x: f32, foot_y: f32, tolerance: f32) -> Option<&LandingPad> {
    for pad in &terrain.pads {
        if x >= pad.x_min && x <= pad.x_max && (foot_y - pad.y_top).abs() <= tolerance {
            return Some(pad);
        }
    }
    None
}

/// Random terrain with `pad_count` flat regions (2–3), shaped by `body`.
pub fn generate_terrain(rng: &mut StdRng, pad_count: usize, body: CelestialBody) -> Terrain {
    let profile = terrain_profile_for(body);
    let half = WORLD_WIDTH * 0.5;
    let x_min = -half;
    let x_max = half;
    let n = TERRAIN_SAMPLES;
    let step = (x_max - x_min) / (n - 1) as f32;

    let mut ys: Vec<f32> = Vec::with_capacity(n);
    let a = profile.sin_amp;
    for i in 0..n {
        let t = i as f32 / (n - 1) as f32;
        let wobble = (t * PI * 11.0).sin() * a[0]
            + (t * PI * 27.0).sin() * a[1]
            + (t * PI * 53.0).sin() * a[2]
            + (t * PI * 91.0).sin() * a[3];
        let noise = rng.gen_range(-profile.noise_half..profile.noise_half);
        let micro = if rng.gen_bool(profile.micro_spike_prob as f64) {
            rng.gen_range(-profile.micro_spike_half..profile.micro_spike_half)
        } else {
            0.0
        };
        let y = profile.base + wobble + noise + micro;
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
        let flat_y = rng.gen_range(profile.pad_y_min..profile.pad_y_max);
        for idx in i0..=i1 {
            ys[idx] = flat_y;
        }
        pads.push(LandingPad {
            x_min: x_min + i0 as f32 * step,
            x_max: x_min + i1 as f32 * step,
            y_top: flat_y,
        });
    }

    let mut volcano_flats: Vec<(LandingPad, f32)> = Vec::new();
    if body == CelestialBody::Io {
        if let Some(z) = try_flatten_volcano_zone(
            rng,
            x_min,
            x_max,
            step,
            n,
            &pads,
            &[],
            &profile,
            &mut ys,
        ) {
            volcano_flats.push(z);
        }
    }
    if body == CelestialBody::Mercury {
        let want = rng.gen_range(1..=2);
        for _ in 0..want {
            let existing: Vec<LandingPad> = volcano_flats.iter().map(|(p, _)| *p).collect();
            if let Some(z) = try_flatten_volcano_zone(
                rng,
                x_min,
                x_max,
                step,
                n,
                &pads,
                &existing,
                &profile,
                &mut ys,
            ) {
                volcano_flats.push(z);
            }
        }
    }

    let mut points: Vec<Vec2> = (0..n)
        .map(|i| Vec2::new(x_min + i as f32 * step, ys[i]))
        .collect();

    for _ in 0..profile.smoothing_passes {
        let copy = points.clone();
        for i in 1..points.len() - 1 {
            let x = points[i].x;
            let on_pad = pads
                .iter()
                .any(|p| x >= p.x_min && x <= p.x_max);
            let on_volcano = volcano_flats
                .iter()
                .any(|(p, _)| x >= p.x_min && x <= p.x_max);
            if !on_pad && !on_volcano {
                points[i].y = copy[i - 1].y * 0.25 + copy[i].y * 0.5 + copy[i + 1].y * 0.25;
            }
        }
    }

    let mut terrain = Terrain {
        points,
        pads,
        io_volcano: None,
        mercury_volcanoes: Vec::new(),
        enceladus_plumes: Vec::new(),
    };

    for (z, height_scale) in &volcano_flats {
        let cx = (z.x_min + z.x_max) * 0.5;
        let vy = z.y_top;
        let vent = VolcanoVent {
            pos: Vec2::new(cx, vy),
            height_scale: *height_scale,
        };
        match body {
            CelestialBody::Io => terrain.io_volcano = Some(vent),
            CelestialBody::Mercury => terrain.mercury_volcanoes.push(vent),
            _ => {}
        }
    }

    // Fallback if placement failed (very crowded random layout): slope placement.
    if body == CelestialBody::Io && terrain.io_volcano.is_none() {
        let vx = pick_x_avoiding_pads(rng, x_min, x_max, &terrain.pads, 50.0);
        let vy = terrain_height_at(&terrain, vx);
        terrain.io_volcano = Some(VolcanoVent {
            pos: Vec2::new(vx, vy),
            height_scale: random_volcano_height_scale(rng),
        });
    }
    if body == CelestialBody::Mercury && terrain.mercury_volcanoes.is_empty() {
        let vx = pick_x_avoiding_pads(rng, x_min, x_max, &terrain.pads, 50.0);
        let vy = terrain_height_at(&terrain, vx);
        terrain.mercury_volcanoes.push(VolcanoVent {
            pos: Vec2::new(vx, vy),
            height_scale: random_volcano_height_scale(rng),
        });
    }

    if body == CelestialBody::Enceladus {
        let n_plumes = rng.gen_range(2..=3);
        for _ in 0..n_plumes {
            let px = pick_x_avoiding_pads(rng, x_min, x_max, &terrain.pads, 42.0);
            let py = terrain_height_at(&terrain, px);
            terrain.enceladus_plumes.push(Vec2::new(px, py));
        }
    }

    terrain
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
    body: CelestialBody,
    fade_in_from_invisible: bool,
) -> Entity {
    let mesh = build_terrain_mesh(terrain);
    let a = if fade_in_from_invisible { 0.0 } else { 1.0 };
    let (r, g, b) = body.terrain_surface_rgb();
    commands
        .spawn((
            TerrainRoot,
            Mesh2d(meshes.add(mesh)),
            MeshMaterial2d(materials.add(Color::srgba(r, g, b, a))),
            Transform::default(),
        ))
        .id()
}
