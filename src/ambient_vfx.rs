//! Io / Mercury volcano meshes + particles; Enceladus cryo-plumes (no volcano).

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use rand::Rng;

use crate::game_flow::{AppState, CurrentBody};
use crate::planets::CelestialBody;
use crate::terrain::{Terrain, VolcanoVent};

#[derive(Component)]
pub struct AmbientVfx;

#[derive(Component)]
pub struct AmbientAirParticle {
    pub age: f32,
    pub max_age: f32,
    /// Vertical drift speed (world units / s); higher for volcanic plumes.
    pub rise_speed: f32,
}

#[derive(Resource, Clone)]
struct AmbientParticleMesh(pub Handle<Mesh>);

fn setup_ambient_mesh(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.insert_resource(AmbientParticleMesh(meshes.add(Circle::new(2.2))));
}

/// Local Y of the crater rim (lowest drawn geometry). Nothing extends below this — avoids “leg” corners.
const VOLCANO_RIM_LOCAL_Y: f32 = 5.0;
/// Apex height above the rim (local space, before Y scale).
const VOLCANO_APEX_LOCAL_Y: f32 = 12.5;
/// Half-width of the rim arc in local units.
const VOLCANO_RIM_HALF_W: f32 = 24.0;

/// Smooth caldera dome: wide curved rim + single apex — no sharp bottom corners (no “lander legs”).
fn volcano_mound_mesh() -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    let segments = 22usize;
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(segments + 3);
    positions.push([0.0, VOLCANO_APEX_LOCAL_Y, 0.0]);
    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let x = -VOLCANO_RIM_HALF_W + t * (2.0 * VOLCANO_RIM_HALF_W);
        positions.push([x, VOLCANO_RIM_LOCAL_Y, 0.0]);
    }
    let mut indices: Vec<u32> = Vec::with_capacity(segments * 3);
    for i in 0..segments {
        indices.extend_from_slice(&[0, (i + 1) as u32, (i + 2) as u32]);
    }
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

fn spawn_volcano_vent(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    (r, g, b): (f32, f32, f32),
    vent: VolcanoVent,
) {
    let mound_h = meshes.add(volcano_mound_mesh());
    // Rim (local y ≈ VOLCANO_RIM_LOCAL_Y) must sit exactly on the recessed plateau in world space:
    // world_y = translation.y + local_y * s  →  translation.y = vent.pos.y - VOLCANO_RIM_LOCAL_Y * s
    // (bow on rim uses small offset; approximate using rim constant.)
    let mound_mat = materials.add(Color::srgba(r, g, b, 0.94));
    let s = vent.height_scale.max(0.5);
    let ty = vent.pos.y - VOLCANO_RIM_LOCAL_Y * s;
    commands.spawn((
        AmbientVfx,
        Mesh2d(mound_h),
        MeshMaterial2d(mound_mat),
        Transform::from_translation(Vec3::new(vent.pos.x, ty, 0.27))
            .with_scale(Vec3::new(1.0, s, 1.0)),
    ));
}

/// Volcano mounds on Io and Mercury (terrain-tinted; base flush with surface).
pub fn spawn_ambient_vfx(
    commands: &mut Commands,
    terrain: &Terrain,
    body: CelestialBody,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) {
    let rgb = body.terrain_surface_rgb();

    if body == CelestialBody::Io {
        let Some(v) = terrain.io_volcano else {
            return;
        };
        spawn_volcano_vent(commands, meshes, materials, rgb, v);
        return;
    }

    if body == CelestialBody::Mercury {
        if terrain.mercury_volcanoes.is_empty() {
            return;
        }
        for v in &terrain.mercury_volcanoes {
            spawn_volcano_vent(commands, meshes, materials, rgb, *v);
        }
    }
}

pub fn despawn_ambient_vfx(commands: &mut Commands, query: &Query<Entity, With<AmbientVfx>>) {
    let entities: Vec<Entity> = query.iter().collect();
    for e in entities {
        commands.entity(e).despawn();
    }
}

fn emit_mercury_particles(
    time: Res<Time>,
    mut commands: Commands,
    mut acc: Local<f32>,
    terrain: Res<Terrain>,
    body: Res<CurrentBody>,
    circle: Res<AmbientParticleMesh>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if body.0 != CelestialBody::Mercury {
        return;
    }
    if terrain.mercury_volcanoes.is_empty() {
        return;
    }
    let (r, g, b) = body.0.terrain_surface_rgb();
    *acc += time.delta_secs();
    if *acc < 0.19 {
        return;
    }
    *acc = 0.0;

    let mut rng = rand::thread_rng();
    for vent in &terrain.mercury_volcanoes {
        let n = rng.gen_range(1..=2);
        for _ in 0..n {
            let ox = rng.gen_range(-4.5..4.5);
            let oy = rng.gen_range(4.0..16.0) * vent.height_scale;
            let world = Vec3::new(vent.pos.x + ox, vent.pos.y + oy, 0.4);
            let max_age = rng.gen_range(1.6..2.85);
            let mat = materials.add(Color::srgba(
                (r * 1.06).min(1.0),
                (g * 1.04).min(1.0),
                (b * 1.05).min(1.0),
                0.58,
            ));
            commands.spawn((
                AmbientAirParticle {
                    age: 0.0,
                    max_age,
                    rise_speed: 82.0,
                },
                AmbientVfx,
                Mesh2d(circle.0.clone()),
                MeshMaterial2d(mat),
                Transform::from_translation(world).with_scale(Vec3::splat(rng.gen_range(0.65..1.05))),
            ));
        }
    }
}

fn emit_io_particles(
    time: Res<Time>,
    mut commands: Commands,
    mut acc: Local<f32>,
    terrain: Res<Terrain>,
    body: Res<CurrentBody>,
    circle: Res<AmbientParticleMesh>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if body.0 != CelestialBody::Io {
        return;
    }
    if terrain.io_volcano.is_none() {
        return;
    }
    let vent = terrain.io_volcano.unwrap();
    let (r, g, b) = body.0.terrain_surface_rgb();
    *acc += time.delta_secs();
    if *acc < 0.17 {
        return;
    }
    *acc = 0.0;

    let mut rng = rand::thread_rng();
    let n = rng.gen_range(1..=2);
    for _ in 0..n {
        let ox = rng.gen_range(-4.5..4.5);
        let oy = rng.gen_range(4.0..16.0) * vent.height_scale;
        let world = Vec3::new(vent.pos.x + ox, vent.pos.y + oy, 0.4);
        let max_age = rng.gen_range(1.65..2.9);
        let mat = materials.add(Color::srgba(
            (r * 1.08).min(1.0),
            (g * 1.05).min(1.0),
            (b * 1.04).min(1.0),
            0.62,
        ));
        commands.spawn((
            AmbientAirParticle {
                age: 0.0,
                max_age,
                rise_speed: 84.0,
            },
            AmbientVfx,
            Mesh2d(circle.0.clone()),
            MeshMaterial2d(mat),
            Transform::from_translation(world).with_scale(Vec3::splat(rng.gen_range(0.7..1.1))),
        ));
    }
}

fn emit_enceladus_particles(
    time: Res<Time>,
    mut commands: Commands,
    mut acc: Local<f32>,
    terrain: Res<Terrain>,
    body: Res<CurrentBody>,
    circle: Res<AmbientParticleMesh>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if body.0 != CelestialBody::Enceladus {
        return;
    }
    if terrain.enceladus_plumes.is_empty() {
        return;
    }
    *acc += time.delta_secs();
    if *acc < 0.1 {
        return;
    }
    *acc = 0.0;

    let mut rng = rand::thread_rng();
    for p in &terrain.enceladus_plumes {
        let ox = rng.gen_range(-3.0..3.0);
        let oy = rng.gen_range(1.5..5.0);
        let world = Vec3::new(p.x + ox, p.y + oy, 0.35);
        let max_age = rng.gen_range(1.0..1.9);
        let mat = materials.add(Color::srgba(0.75, 0.92, 1.0, 0.55));
        commands.spawn((
            AmbientAirParticle {
                age: 0.0,
                max_age,
                rise_speed: 30.0,
            },
            AmbientVfx,
            Mesh2d(circle.0.clone()),
            MeshMaterial2d(mat),
            Transform::from_translation(world).with_scale(Vec3::splat(rng.gen_range(0.65..1.05))),
        ));
    }
}

fn tick_ambient_particles(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut AmbientAirParticle, &mut Transform), With<AmbientAirParticle>>,
) {
    let dt = time.delta_secs();
    for (entity, mut p, mut tf) in &mut q {
        p.age += dt;
        tf.translation.y += p.rise_speed * dt;
        let t = (p.age / p.max_age).min(1.0);
        let s = (1.0 - t).max(0.0);
        tf.scale = Vec3::splat(s * 0.95 + 0.05);
        if p.age >= p.max_age {
            commands.entity(entity).despawn();
        }
    }
}

pub struct AmbientVfxPlugin;

impl Plugin for AmbientVfxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ambient_mesh)
            .add_systems(
                Update,
                (
                    emit_io_particles
                        .run_if(in_state(AppState::Playing)),
                    emit_mercury_particles
                        .run_if(in_state(AppState::Playing)),
                    emit_enceladus_particles
                        .run_if(in_state(AppState::Playing)),
                    tick_ambient_particles,
                ),
            );
    }
}
