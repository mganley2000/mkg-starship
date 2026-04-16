//! Short-lived exhaust puffs when thrusters fire.

use bevy::prelude::*;
use rand::Rng;

use crate::game_flow::AppState;
use crate::ship::{Ship, ShipRoot, thruster_local_offsets};

#[derive(Component)]
pub struct ExhaustPlume {
    pub age: f32,
    pub max_age: f32,
}

#[derive(Resource, Clone)]
pub struct ExhaustCircleMesh(pub Handle<Mesh>);

fn setup_exhaust_mesh(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let h = meshes.add(Circle::new(4.5));
    commands.insert_resource(ExhaustCircleMesh(h));
}

fn spawn_exhaust(
    keyboard: Res<ButtonInput<KeyCode>>,
    ship_q: Query<(&Transform, &Ship), With<ShipRoot>>,
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    circle: Res<ExhaustCircleMesh>,
    mut acc: Local<f32>,
    time: Res<Time>,
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

    let mut spawn_one = |local: Vec3| {
        let jitter = Vec3::new(rng.gen_range(-2.5..2.5), rng.gen_range(-2.0..1.5), 0.0);
        let world = tf.translation + local + jitter;
        let max_age = rng.gen_range(1.2..2.4);
        let mat = materials.add(Color::srgba(0.42, 0.78, 1.0, 0.82));
        commands.spawn((
            ExhaustPlume { age: 0.0, max_age },
            Mesh2d(circle.0.clone()),
            MeshMaterial2d(mat),
            Transform::from_translation(world).with_scale(Vec3::splat(rng.gen_range(0.85..1.35))),
        ));
    };

    if main {
        spawn_one(o_main);
    }
    if left {
        spawn_one(o_left);
    }
    if right {
        spawn_one(o_right);
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
            let a = (1.0 - t) * 0.85;
            // Methane: cool blue → slightly deeper blue as the plume ages.
            mat.color = Color::srgba(0.32 + t * 0.12, 0.68 + t * 0.08, 1.0 - t * 0.12, a);
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
