//! Regolith dust kicked up when the ship is near the ground. Lifetime and count scale with
//! real surface gravity: low g → more particles, longer-lived; high g → fewer, very short-lived.

use bevy::prelude::*;
use rand::Rng;

use crate::constants::LANDING_DUST_ALTITUDE_METERS;
use crate::game_flow::{AppState, CurrentBody};
use crate::planets::{game_vertical_distance_to_meters, gravity_acceleration};
use crate::ship::{Ship, ShipRoot, ship_foot_y, ship_left_foot_x, ship_right_foot_x};
use crate::terrain::{Terrain, terrain_height_at};

const EMIT_INTERVAL_SECS: f32 = 0.065;

#[derive(Component)]
struct LandingDustParticle {
    age: f32,
    max_age: f32,
    vel: Vec2,
    /// RGB matched to `CelestialBody::terrain_surface_rgb` (small jitter per particle).
    rgb: [f32; 3],
}

#[derive(Component)]
struct LandingDust;

#[derive(Resource, Clone)]
struct LandingDustMesh(Handle<Mesh>);

fn setup_landing_dust_mesh(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.insert_resource(LandingDustMesh(meshes.add(Circle::new(2.0))));
}

/// Returns (max_age_secs, particle_count) from real gravity g (m/s²).
fn dust_params(g_real: f32) -> (f32, usize) {
    const G_EARTH: f32 = 9.81;
    let ratio = (G_EARTH / g_real).clamp(0.18, 32.0);
    let max_age = (0.11 * ratio.powf(0.58)).clamp(0.05, 2.15);
    let n = (3.0 + 7.0 * ratio.powf(0.4))
        .clamp(2.0, 18.0)
        .round() as usize;
    (max_age, n.max(2))
}

fn emit_landing_dust(
    time: Res<Time>,
    mut commands: Commands,
    mut acc: Local<f32>,
    terrain: Res<Terrain>,
    body: Res<CurrentBody>,
    ship_q: Query<(&Transform, &Ship), With<ShipRoot>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mesh: Res<LandingDustMesh>,
) {
    *acc += time.delta_secs();
    if *acc < EMIT_INTERVAL_SECS {
        return;
    }
    *acc = 0.0;

    let Ok((tf, _ship)) = ship_q.single() else {
        return;
    };
    if terrain.points.is_empty() {
        return;
    }

    let xl = ship_left_foot_x(tf);
    let xr = ship_right_foot_x(tf);
    let foot_y = ship_foot_y(tf);
    let h_l = terrain_height_at(&terrain, xl);
    let h_r = terrain_height_at(&terrain, xr);
    let ground_ref = h_l.min(h_r);

    let clearance_game = foot_y - ground_ref;
    if clearance_game <= 0.0 {
        return;
    }

    if game_vertical_distance_to_meters(clearance_game) > LANDING_DUST_ALTITUDE_METERS {
        return;
    }

    let g_real = body.0.real_gravity_m_s2();
    let (max_age, count) = dust_params(g_real);
    let (tr, tg, tb) = body.0.terrain_surface_rgb();

    let mut rng = rand::thread_rng();
    let cx = (xl + xr) * 0.5;
    let spread = (xr - xl).max(28.0);

    for _ in 0..count {
        let ox = rng.gen_range(-spread * 0.55..spread * 0.55);
        let px = cx + ox;
        let py = ground_ref + rng.gen_range(0.5..5.0);

        // Kick mostly upward and sideways (regolith spray).
        let mut vy = rng.gen_range(32.0..115.0);
        let vx = rng.gen_range(-78.0..78.0);
        // High gravity: lower initial kick so dust stays near the surface.
        vy *= (0.55 + 9.81 / (g_real + 6.0)).clamp(0.45, 1.05);

        // Dust color: terrain tint with tiny per-particle jitter (stay very close to surface).
        let r = (tr * rng.gen_range(0.96..1.04)).clamp(0.0, 1.0);
        let g = (tg * rng.gen_range(0.96..1.04)).clamp(0.0, 1.0);
        let b = (tb * rng.gen_range(0.96..1.04)).clamp(0.0, 1.0);
        let a = rng.gen_range(0.35..0.58);
        let mat = materials.add(Color::srgba(r, g, b, a));

        commands.spawn((
            LandingDust,
            LandingDustParticle {
                age: 0.0,
                max_age,
                vel: Vec2::new(vx, vy),
                rgb: [r, g, b],
            },
            Mesh2d(mesh.0.clone()),
            MeshMaterial2d(mat),
            Transform::from_translation(Vec3::new(px, py, 0.18))
                .with_scale(Vec3::splat(rng.gen_range(0.75..1.15))),
        ));
    }
}

fn tick_landing_dust(
    time: Res<Time>,
    mut commands: Commands,
    body: Res<CurrentBody>,
    mut q: Query<(
        Entity,
        &mut LandingDustParticle,
        &mut Transform,
        &MeshMaterial2d<ColorMaterial>,
    )>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let dt = time.delta_secs();
    let g = gravity_acceleration(body.0);

    for (entity, mut p, mut tf, mat_wrap) in &mut q {
        p.age += dt;
        p.vel.y -= g * dt;
        p.vel *= 0.998;
        tf.translation.x += p.vel.x * dt;
        tf.translation.y += p.vel.y * dt;

        let t = (p.age / p.max_age).min(1.0);
        if let Some(mat) = materials.get_mut(mat_wrap.id()) {
            let a = (1.0 - t).powf(0.9) * 0.58;
            let [r, g, b] = p.rgb;
            mat.color = Color::srgba(r, g, b, a);
        }

        if p.age >= p.max_age {
            commands.entity(entity).despawn();
        }
    }
}

pub struct LandingDustPlugin;

impl Plugin for LandingDustPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_landing_dust_mesh).add_systems(
            Update,
            (
                emit_landing_dust.run_if(in_state(AppState::Playing)),
                tick_landing_dust,
            ),
        );
    }
}
