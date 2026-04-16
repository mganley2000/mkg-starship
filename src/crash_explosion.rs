//! Crash: fireball, blue/red burst particles, then charred wreckage.

use bevy::prelude::*;
use rand::Rng;

use crate::game_flow::CrashEvent;

/// Marks entities spawned for a crash so they can be cleared on restart.
#[derive(Component)]
pub struct CrashFx;

#[derive(Component)]
struct CrashFireball {
    age: f32,
}

/// Brighter inner core of the fireball (same timing as outer).
#[derive(Component)]
struct CrashFireballCore {
    age: f32,
}

#[derive(Component)]
struct CrashBurstParticle {
    age: f32,
    max_age: f32,
    vel: Vec2,
}

#[derive(Component)]
struct CrashWreckChunk {
    age: f32,
    vel: Vec2,
    spin: f32,
}

#[derive(Resource, Clone)]
struct CrashMeshes {
    particle: Handle<Mesh>,
    wreck_rect: Handle<Mesh>,
}

const FIREBALL_MAX_SEC: f32 = 1.35;
const BURST_MAX_SEC: f32 = 1.85;
const WRECK_MAX_SEC: f32 = 6.5;

fn setup_crash_meshes(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.insert_resource(CrashMeshes {
        particle: meshes.add(Circle::new(3.2)),
        wreck_rect: meshes.add(Rectangle::new(1.0, 1.0)),
    });
}

fn spawn_crash_effects(
    mut reader: MessageReader<CrashEvent>,
    mut commands: Commands,
    ship_q: Query<Entity, With<crate::ship::ShipRoot>>,
    meshes: Res<CrashMeshes>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for ev in reader.read() {
        let o = ev.origin;
        let world = Vec3::new(o.x, o.y, 0.55);

        if let Ok(e) = ship_q.single() {
            // Hierarchy: despawning the root removes child meshes.
            commands.entity(e).despawn();
        }

        let fire_mat = materials.add(Color::srgba(1.0, 0.42, 0.1, 0.88));
        commands.spawn((
            CrashFx,
            CrashFireball { age: 0.0 },
            Mesh2d(meshes.particle.clone()),
            MeshMaterial2d(fire_mat),
            Transform::from_translation(world).with_scale(Vec3::splat(18.0)),
        ));
        let core_mat = materials.add(Color::srgba(1.0, 0.92, 0.45, 0.75));
        commands.spawn((
            CrashFx,
            CrashFireballCore { age: 0.0 },
            Mesh2d(meshes.particle.clone()),
            MeshMaterial2d(core_mat),
            Transform::from_translation(Vec3::new(o.x, o.y, 0.56)).with_scale(Vec3::splat(9.0)),
        ));

        let mut rng = rand::thread_rng();

        for i in 0..44 {
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
            let speed = rng.gen_range(95.0..240.0);
            let vel = Vec2::new(
                angle.cos() * speed,
                angle.sin() * speed + rng.gen_range(20.0..90.0),
            );
            let is_blue = i % 2 == 0;
            let (r, g, b) = if is_blue {
                (0.22, 0.55, 1.0)
            } else {
                (1.0, 0.18, 0.14)
            };
            let max_age = rng.gen_range(1.2..BURST_MAX_SEC);
            let mat = materials.add(Color::srgba(r, g, b, 0.92));
            let jitter = Vec2::new(rng.gen_range(-6.0..6.0), rng.gen_range(-4.0..8.0));
            commands.spawn((
                CrashFx,
                CrashBurstParticle {
                    age: 0.0,
                    max_age,
                    vel,
                },
                Mesh2d(meshes.particle.clone()),
                MeshMaterial2d(mat),
                Transform::from_translation(Vec3::new(o.x + jitter.x, o.y + jitter.y, 0.48))
                    .with_scale(Vec3::splat(rng.gen_range(0.75..1.25))),
            ));
        }

        for _ in 0..20 {
            let ang = rng.gen_range(0.0..std::f32::consts::TAU);
            let dist = rng.gen_range(4.0..28.0);
            let ox = ang.cos() * dist + rng.gen_range(-5.0..5.0);
            let oy = ang.sin() * dist * 0.35 + rng.gen_range(-3.0..12.0);
            let w = rng.gen_range(2.5..7.0);
            let h = rng.gen_range(1.2..4.5);
            let mat = materials.add(Color::srgba(0.04, 0.04, 0.05, 0.96));
            let spin = rng.gen_range(-4.5..4.5);
            let v = Vec2::new(
                rng.gen_range(-35.0..35.0) + ang.cos() * 25.0,
                rng.gen_range(15.0..85.0) + ang.sin() * 15.0,
            );
            commands.spawn((
                CrashFx,
                CrashWreckChunk {
                    age: 0.0,
                    vel: v,
                    spin,
                },
                Mesh2d(meshes.wreck_rect.clone()),
                MeshMaterial2d(mat),
                Transform::from_translation(Vec3::new(o.x + ox, o.y + oy, 0.35))
                    .with_scale(Vec3::new(w, h, 1.0))
                    .with_rotation(Quat::from_rotation_z(rng.gen_range(0.0..std::f32::consts::TAU))),
            ));
        }
    }
}

fn tick_crash_fireball(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut CrashFireball, &mut Transform, &MeshMaterial2d<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let dt = time.delta_secs();
    for (entity, mut fb, mut tf, mat_wrap) in &mut q {
        fb.age += dt;
        let t = (fb.age / FIREBALL_MAX_SEC).min(1.0);
        let s = 18.0 + t * 102.0;
        tf.scale = Vec3::splat(s);
        tf.translation.y += 22.0 * dt;
        if let Some(mat) = materials.get_mut(mat_wrap.id()) {
            let a = (1.0 - t).powf(1.15) * 0.88;
            mat.color = Color::srgba(1.0, 0.32 + t * 0.25, 0.05 + t * 0.08, a);
        }
        if fb.age >= FIREBALL_MAX_SEC {
            commands.entity(entity).despawn();
        }
    }
}

fn tick_crash_fireball_core(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut CrashFireballCore, &mut Transform, &MeshMaterial2d<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let dt = time.delta_secs();
    for (entity, mut fb, mut tf, mat_wrap) in &mut q {
        fb.age += dt;
        let t = (fb.age / FIREBALL_MAX_SEC).min(1.0);
        let s = 9.0 + t * 58.0;
        tf.scale = Vec3::splat(s);
        tf.translation.y += 22.0 * dt;
        if let Some(mat) = materials.get_mut(mat_wrap.id()) {
            let a = (1.0 - t).powf(1.1) * 0.75;
            mat.color = Color::srgba(1.0, 0.88 - t * 0.35, 0.2 + t * 0.15, a);
        }
        if fb.age >= FIREBALL_MAX_SEC {
            commands.entity(entity).despawn();
        }
    }
}

fn tick_crash_burst(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(
        Entity,
        &mut CrashBurstParticle,
        &mut Transform,
        &MeshMaterial2d<ColorMaterial>,
    )>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let dt = time.delta_secs();
    for (entity, mut p, mut tf, mat_wrap) in &mut q {
        p.age += dt;
        p.vel *= 0.985;
        p.vel.y -= 45.0 * dt;
        tf.translation.x += p.vel.x * dt;
        tf.translation.y += p.vel.y * dt;
        let t = (p.age / p.max_age).min(1.0);
        if let Some(mat) = materials.get_mut(mat_wrap.id()) {
            let a = (1.0 - t).powf(0.9) * 0.92;
            mat.color = mat.color.with_alpha(a);
        }
        if p.age >= p.max_age {
            commands.entity(entity).despawn();
        }
    }
}

fn tick_crash_wreck(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(
        Entity,
        &mut CrashWreckChunk,
        &mut Transform,
        &MeshMaterial2d<ColorMaterial>,
    )>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let dt = time.delta_secs();
    for (entity, mut w, mut tf, mat_wrap) in &mut q {
        w.age += dt;
        w.vel.y -= 120.0 * dt;
        w.vel *= 0.992;
        tf.translation.x += w.vel.x * dt;
        tf.translation.y += w.vel.y * dt;
        let z = tf.rotation.to_euler(EulerRot::ZYX).0 + w.spin * dt;
        tf.rotation = Quat::from_rotation_z(z);

        let fade_start = 2.2_f32;
        if w.age > fade_start {
            let u = ((w.age - fade_start) / (WRECK_MAX_SEC - fade_start)).min(1.0);
            if let Some(mat) = materials.get_mut(mat_wrap.id()) {
                let a = (1.0 - u) * 0.96;
                mat.color = Color::srgba(0.04, 0.04, 0.05, a);
            }
        }
        if w.age >= WRECK_MAX_SEC {
            commands.entity(entity).despawn();
        }
    }
}

pub struct CrashExplosionPlugin;

impl Plugin for CrashExplosionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_crash_meshes).add_systems(
            Update,
            (
                spawn_crash_effects,
                tick_crash_fireball,
                tick_crash_fireball_core,
                tick_crash_burst,
                tick_crash_wreck,
            ),
        );
    }
}
