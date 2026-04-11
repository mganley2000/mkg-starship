//! App state, planet progression, score, landing / crash handling, restart.

use bevy::prelude::*;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::constants::{INITIAL_FUEL, ship_spawn_y};
use crate::planets::Planet;
use crate::ship::{Ship, ShipRoot};
use crate::terrain::{Terrain, TerrainRoot, generate_terrain, spawn_terrain_entity};

#[derive(States, Clone, Copy, Default, Eq, PartialEq, Hash, Debug)]
pub enum AppState {
    #[default]
    Playing,
    GameOver,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EndReason {
    Crashed,
    Victory,
}

#[derive(Resource)]
pub struct Score(pub i32);

#[derive(Resource)]
pub struct CurrentPlanet(pub Planet);

#[derive(Resource)]
pub struct GameEnd {
    pub reason: EndReason,
    pub score: i32,
}

#[derive(Resource)]
pub struct GameRng(pub StdRng);

#[derive(Message, Clone)]
pub struct LandedOnPad {
    pub total_score: i32,
}

#[derive(Message)]
pub struct CrashEvent;

pub struct GameFlowPlugin;

impl Plugin for GameFlowPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .add_message::<LandedOnPad>()
            .add_message::<CrashEvent>()
            .insert_resource(Score(0))
            .insert_resource(CurrentPlanet(Planet::ORDER[0]))
            .insert_resource(GameRng(StdRng::from_entropy()))
            .insert_resource(GameEnd {
                reason: EndReason::Crashed,
                score: 0,
            })
            .insert_resource(Terrain {
                points: vec![],
                pads: vec![],
            })
            .add_systems(Startup, setup_world)
            .add_systems(
                FixedUpdate,
                (
                    crate::physics::ship_physics,
                    crate::collision::ground_contact,
                    handle_ground_events,
                )
                    .chain()
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(Update, restart_input);
    }
}

fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut terrain_res: ResMut<Terrain>,
    mut rng: ResMut<GameRng>,
) {
    let pad_count = rng.0.gen_range(2..=3);
    *terrain_res = generate_terrain(&mut rng.0, pad_count);
    spawn_terrain_entity(&mut commands, &mut meshes, &mut materials, &terrain_res);
    crate::ship::spawn_ship(&mut commands, &mut meshes, &mut materials);
}

fn handle_ground_events(
    mut landed: MessageReader<LandedOnPad>,
    mut crashed: MessageReader<CrashEvent>,
    mut next: ResMut<NextState<AppState>>,
    mut planet: ResMut<CurrentPlanet>,
    mut terrain_res: ResMut<Terrain>,
    mut game_end: ResMut<GameEnd>,
    score: Res<Score>,
    mut rng: ResMut<GameRng>,
    mut commands: Commands,
    terrain_entities: Query<Entity, With<TerrainRoot>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut ship_q: Query<(&mut Ship, &mut Transform), With<ShipRoot>>,
) {
    for _ in crashed.read() {
        game_end.reason = EndReason::Crashed;
        game_end.score = score.0;
        next.set(AppState::GameOver);
    }

    for ev in landed.read() {
        if planet.0 == Planet::Mercury {
            game_end.reason = EndReason::Victory;
            game_end.score = ev.total_score;
            next.set(AppState::GameOver);
            continue;
        }

        if let Some(next_planet) = planet.0.next() {
            planet.0 = next_planet;
        }

        for e in &terrain_entities {
            commands.entity(e).despawn();
        }

        let pad_count = rng.0.gen_range(2..=3);
        *terrain_res = generate_terrain(&mut rng.0, pad_count);
        spawn_terrain_entity(&mut commands, &mut meshes, &mut materials, &terrain_res);

        for (mut ship, mut tf) in &mut ship_q {
            ship.velocity = Vec2::ZERO;
            ship.fuel = INITIAL_FUEL;
            tf.translation = Vec3::new(0.0, ship_spawn_y(), 1.0);
        }
    }
}

fn restart_input(
    state: Res<State<AppState>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next: ResMut<NextState<AppState>>,
    mut score: ResMut<Score>,
    mut planet: ResMut<CurrentPlanet>,
    mut terrain_res: ResMut<Terrain>,
    mut rng: ResMut<GameRng>,
    mut commands: Commands,
    terrain_entities: Query<Entity, With<TerrainRoot>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut ship_q: Query<(&mut Ship, &mut Transform), With<ShipRoot>>,
) {
    if *state.get() != AppState::GameOver {
        return;
    }
    if !keyboard.just_pressed(KeyCode::KeyR) {
        return;
    }

    score.0 = 0;
    planet.0 = Planet::ORDER[0];

    for e in &terrain_entities {
        commands.entity(e).despawn();
    }

    let pad_count = rng.0.gen_range(2..=3);
    *terrain_res = generate_terrain(&mut rng.0, pad_count);
    spawn_terrain_entity(&mut commands, &mut meshes, &mut materials, &terrain_res);

    for (mut ship, mut tf) in &mut ship_q {
        ship.velocity = Vec2::ZERO;
        ship.fuel = INITIAL_FUEL;
        tf.translation = Vec3::new(0.0, ship_spawn_y(), 1.0);
    }

    next.set(AppState::Playing);
}
