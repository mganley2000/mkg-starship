//! Short-lived exhaust puffs when thrusters fire.

use bevy::prelude::*;
use rand::Rng;

use crate::game_flow::{AppState, CurrentBody};
use crate::planets::CelestialBody;
use crate::ship::{Ship, ShipRoot, thruster_local_offsets};

#[derive(Component, Clone, Copy)]
enum ExhaustStyle {
    /// Blue outer halo (methane).
    MethaneHalo,
    /// White core (methane).
    MethaneCore,
    /// Outer purple halo (Jupiter plasma).
    PlasmaHalo,
    /// Inner black core.
    PlasmaCore,
}

#[derive(Component)]
pub struct ExhaustPlume {
    pub age: f32,
    pub max_age: f32,
    style: ExhaustStyle,
}

#[derive(Resource, Clone)]
struct ExhaustMeshes {
    methane_halo: Handle<Mesh>,
    methane_core: Handle<Mesh>,
    plasma_halo: Handle<Mesh>,
    plasma_core: Handle<Mesh>,
}

fn setup_exhaust_mesh(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.insert_resource(ExhaustMeshes {
        methane_halo: meshes.add(Circle::new(7.0)),
        methane_core: meshes.add(Circle::new(4.0)),
        plasma_halo: meshes.add(Circle::new(7.0)),
        plasma_core: meshes.add(Circle::new(4.0)),
    });
}

fn spawn_exhaust(
    keyboard: Res<ButtonInput<KeyCode>>,
    ship_q: Query<(&Transform, &Ship), With<ShipRoot>>,
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    meshes: Res<ExhaustMeshes>,
    mut acc: Local<f32>,
    time: Res<Time>,
    body: Res<CurrentBody>,
) {
    let Ok((tf, ship)) = ship_q.single() else {
        return;
    };
    if ship.fuel <= 0.0 {
        return;
    }

    let main = keyboard.pressed(KeyCode::ArrowDown);
    let left = keyboard.pressed(KeyCode::ArrowLeft);
    let right = keyboard.pressed(KeyCode::ArrowRight);
    if !main && !left && !right {
        return;
    }

    *acc += time.delta_secs();
    if *acc < 0.045 {
        return;
    }
    *acc = 0.0;

    let (o_main, o_left, o_right) = thruster_local_offsets();
    let mut rng = rand::thread_rng();

    if body.0 == CelestialBody::Jupiter {
        let mut spawn_plasma = |local: Vec3| {
            let jitter = Vec3::new(rng.gen_range(-2.5..2.5), rng.gen_range(-2.0..1.5), 0.0);
            let base = tf.translation + local + jitter;
            let max_age = rng.gen_range(1.15..2.1);
            let s_halo = rng.gen_range(0.9..1.25);
            let s_core = rng.gen_range(0.55..0.85);

            let halo_mat = materials.add(Color::srgba(0.52, 0.1, 0.82, 0.48));
            commands.spawn((
                ExhaustPlume {
                    age: 0.0,
                    max_age,
                    style: ExhaustStyle::PlasmaHalo,
                },
                Mesh2d(meshes.plasma_halo.clone()),
                MeshMaterial2d(halo_mat),
                Transform::from_translation(Vec3::new(base.x, base.y, 0.04)).with_scale(
                    Vec3::splat(s_halo),
                ),
            ));

            let core_mat = materials.add(Color::srgba(0.02, 0.02, 0.03, 0.94));
            commands.spawn((
                ExhaustPlume {
                    age: 0.0,
                    max_age,
                    style: ExhaustStyle::PlasmaCore,
                },
                Mesh2d(meshes.plasma_core.clone()),
                MeshMaterial2d(core_mat),
                Transform::from_translation(Vec3::new(base.x, base.y, 0.12)).with_scale(
                    Vec3::splat(s_core),
                ),
            ));
        };

        if main {
            spawn_plasma(o_main);
        }
        if left {
            spawn_plasma(o_left);
        }
        if right {
            spawn_plasma(o_right);
        }
    } else {
        let mut spawn_methane = |local: Vec3| {
            let jitter = Vec3::new(rng.gen_range(-2.5..2.5), rng.gen_range(-2.0..1.5), 0.0);
            let base = tf.translation + local + jitter;
            let max_age = rng.gen_range(1.2..2.4);
            let s_halo = rng.gen_range(0.9..1.25);
            let s_core = rng.gen_range(0.55..0.85);

            let halo_mat = materials.add(Color::srgba(0.28, 0.65, 1.0, 0.52));
            commands.spawn((
                ExhaustPlume {
                    age: 0.0,
                    max_age,
                    style: ExhaustStyle::MethaneHalo,
                },
                Mesh2d(meshes.methane_halo.clone()),
                MeshMaterial2d(halo_mat),
                Transform::from_translation(Vec3::new(base.x, base.y, 0.04)).with_scale(
                    Vec3::splat(s_halo),
                ),
            ));

            let core_mat = materials.add(Color::srgba(0.96, 0.97, 1.0, 0.92));
            commands.spawn((
                ExhaustPlume {
                    age: 0.0,
                    max_age,
                    style: ExhaustStyle::MethaneCore,
                },
                Mesh2d(meshes.methane_core.clone()),
                MeshMaterial2d(core_mat),
                Transform::from_translation(Vec3::new(base.x, base.y, 0.12)).with_scale(
                    Vec3::splat(s_core),
                ),
            ));
        };

        if main {
            spawn_methane(o_main);
        }
        if left {
            spawn_methane(o_left);
        }
        if right {
            spawn_methane(o_right);
        }
    }
}

fn tick_exhaust_plumes(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut ExhaustPlume, &MeshMaterial2d<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let dt = time.delta_secs();
    for (entity, mut plume, mat_wrap) in &mut q {
        plume.age += dt;
        let t = (plume.age / plume.max_age).min(1.0);
        if let Some(mat) = materials.get_mut(mat_wrap.id()) {
            match plume.style {
                ExhaustStyle::MethaneHalo => {
                    let edge = (1.0 - t).powf(1.1);
                    mat.color = Color::srgba(
                        0.22 + t * 0.15,
                        0.55 + t * 0.12,
                        0.95 + t * 0.05,
                        edge * 0.58,
                    );
                }
                ExhaustStyle::MethaneCore => {
                    let fade = (1.0 - t).powf(0.85);
                    mat.color = Color::srgba(
                        0.92 + t * 0.06,
                        0.94 + t * 0.04,
                        1.0,
                        fade * 0.92,
                    );
                }
                ExhaustStyle::PlasmaHalo => {
                    // Purple ring fades outward in opacity; slight brighten then fade.
                    let edge = (1.0 - t).powf(1.15);
                    mat.color = Color::srgba(
                        0.45 + t * 0.25,
                        0.06 + t * 0.08,
                        0.72 + t * 0.15,
                        edge * 0.55,
                    );
                }
                ExhaustStyle::PlasmaCore => {
                    mat.color = Color::srgba(0.015, 0.015, 0.025, (1.0 - t).powf(0.9) * 0.94);
                }
            }
        }
        if plume.age >= plume.max_age {
            commands.entity(entity).despawn();
        }
    }
}

pub struct ExhaustPlugin;

impl Plugin for ExhaustPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_exhaust_mesh)
            .add_systems(
                Update,
                spawn_exhaust.run_if(in_state(AppState::Playing)),
            )
            .add_systems(Update, tick_exhaust_plumes);
    }
}
