//! App state, planet progression, score, landing / crash handling, restart.

use bevy::prelude::*;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::ambient_vfx::{despawn_ambient_vfx, spawn_ambient_vfx};
use crate::crash_explosion::CrashFx;
use crate::constants::{
    EARTH_WATER_ALPHA, EARTH_WATER_RGB, INITIAL_FUEL, SHIP_FOOT_OFFSET_Y,
    SHIP_HULL_BOTTOM_OFFSET_Y, TERRAIN_PARALLAX_FAR_ALPHA,
    ship_spawn_y,
};
use crate::planets::CelestialBody;
use crate::ship::{Ship, ShipRoot};
use crate::persistence;
use crate::terrain::{
    EarthWaterFill, Terrain, TerrainParallaxLayer, TerrainRoot, TerrainStack, generate_terrain,
    spawn_terrain_entity,
};

#[derive(States, Clone, Copy, Default, Eq, PartialEq, Hash, Debug)]
pub enum AppState {
    /// Terrain fade-in, “Get Ready”, then gameplay.
    #[default]
    GetReady,
    Playing,
    /// Brief “Success” after a good landing before loading the next body.
    LandingSuccess,
    GameOver,
}

/// UI markers for the intro overlay (spawned from `ui.rs`).
#[derive(Component)]
pub struct GetReadyRoot;

#[derive(Component)]
pub struct GetReadyBodyName;

#[derive(Component)]
pub struct GetReadyText;

#[derive(Component)]
pub struct SuccessRoot;

#[derive(Component)]
pub struct SuccessText;

/// Elapsed time in the intro sequence while in [`AppState::GetReady`].
#[derive(Resource)]
pub struct IntroTimer {
    pub elapsed: f32,
}

/// How long “Success” stays on screen before the next level intro (seconds).
pub const SUCCESS_DISPLAY_SECS: f32 = 2.0;

/// Elapsed time while showing the success overlay; `transitioned` avoids re-running the level load.
#[derive(Resource)]
pub struct SuccessOverlayTimer {
    pub elapsed: f32,
    pub transitioned: bool,
}

impl Default for SuccessOverlayTimer {
    fn default() -> Self {
        Self {
            elapsed: 0.0,
            transitioned: false,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EndReason {
    Crashed,
    Victory,
}

#[derive(Resource)]
pub struct Score(pub i32);

/// Seconds since this level entered [`AppState::Playing`] (resets each level).
#[derive(Resource)]
pub struct LevelFlightTimer {
    pub elapsed: f32,
}

/// Best runs persisted (see `persistence`); updated when a run ends.
#[derive(Resource)]
pub struct HighScores(pub Vec<crate::persistence::HighScoreEntry>);

#[derive(Resource)]
pub struct CurrentBody(pub CelestialBody);

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

#[derive(Message, Clone)]
pub struct CrashEvent {
    /// World position at the feet / impact (surface anchor for the explosion).
    pub origin: Vec2,
}

pub struct GameFlowPlugin;

impl Plugin for GameFlowPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .add_message::<LandedOnPad>()
            .add_message::<CrashEvent>()
            .insert_resource(Score(0))
            .insert_resource(LevelFlightTimer { elapsed: 0.0 })
            .insert_resource(HighScores(persistence::load_high_scores()))
            .insert_resource(CurrentBody(CelestialBody::ORDER[0]))
            .insert_resource(GameRng(StdRng::from_entropy()))
            .insert_resource(GameEnd {
                reason: EndReason::Crashed,
                score: 0,
            })
            .insert_resource(IntroTimer { elapsed: 0.0 })
            .insert_resource(SuccessOverlayTimer::default())
            .insert_resource(Terrain {
                points: vec![],
                pads: vec![],
                io_volcano: None,
                mercury_volcanoes: vec![],
                enceladus_plumes: vec![],
            })
            .add_systems(Startup, setup_world)
            .add_systems(OnEnter(AppState::GetReady), reset_intro_timer)
            .add_systems(OnEnter(AppState::Playing), reset_level_flight_timer)
            .add_systems(OnEnter(AppState::LandingSuccess), reset_success_timer)
            .add_systems(
                Update,
                (
                    intro_tick.run_if(in_state(AppState::GetReady)),
                    landing_success_tick.run_if(in_state(AppState::LandingSuccess)),
                    tick_level_flight_timer.run_if(in_state(AppState::Playing)),
                ),
            )
            // Ship + collision run in Update (not FixedUpdate) so thrust matches each frame’s
            // input and display — avoids 0..N fixed steps per frame feeling buffered or mushy.
            .add_systems(
                Update,
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

fn reset_intro_timer(mut timer: ResMut<IntroTimer>) {
    timer.elapsed = 0.0;
}

fn reset_level_flight_timer(mut timer: ResMut<LevelFlightTimer>) {
    timer.elapsed = 0.0;
}

fn tick_level_flight_timer(time: Res<Time>, mut timer: ResMut<LevelFlightTimer>) {
    timer.elapsed += time.delta_secs();
}

fn reset_success_timer(mut timer: ResMut<SuccessOverlayTimer>) {
    timer.elapsed = 0.0;
    timer.transitioned = false;
}

fn reset_ship_at_spawn(ship: &mut Ship, tf: &mut Transform) {
    let y = ship_spawn_y();
    tf.translation = Vec3::new(0.0, y, 1.0);
    ship.velocity = Vec2::ZERO;
    ship.fuel = INITIAL_FUEL;
    ship.foot_prev = y + SHIP_FOOT_OFFSET_Y;
    ship.hull_bottom_prev = y + SHIP_HULL_BOTTOM_OFFSET_Y;
}

/// Despawn old terrain/ambient, build next body’s terrain (fade-in for intro), reset ship at spawn.
fn transition_to_next_level(
    commands: &mut Commands,
    terrain_res: &mut Terrain,
    body: CelestialBody,
    rng: &mut GameRng,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    terrain_entities: &Query<Entity, With<TerrainStack>>,
    ambient_entities: &Query<Entity, With<crate::ambient_vfx::AmbientVfx>>,
    ship_q: &mut Query<(&mut Ship, &mut Transform), With<ShipRoot>>,
) {
    despawn_ambient_vfx(commands, ambient_entities);
    for e in terrain_entities.iter() {
        commands.entity(e).despawn();
    }
    let pad_count = rng.0.gen_range(2..=3);
    *terrain_res = generate_terrain(&mut rng.0, pad_count, body);
    spawn_terrain_entity(commands, meshes, materials, terrain_res, body, &mut rng.0, true);
    spawn_ambient_vfx(commands, terrain_res, body, meshes, materials);
    for (mut ship, mut tf) in ship_q.iter_mut() {
        reset_ship_at_spawn(&mut ship, &mut tf);
    }
}

fn landing_success_tick(
    mut timer: ResMut<SuccessOverlayTimer>,
    time: Res<Time>,
    mut next: ResMut<NextState<AppState>>,
    mut terrain_res: ResMut<Terrain>,
    body: Res<CurrentBody>,
    mut rng: ResMut<GameRng>,
    mut commands: Commands,
    terrain_entities: Query<Entity, With<TerrainStack>>,
    ambient_entities: Query<Entity, With<crate::ambient_vfx::AmbientVfx>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut ship_q: Query<(&mut Ship, &mut Transform), With<ShipRoot>>,
) {
    if timer.transitioned {
        return;
    }
    timer.elapsed += time.delta_secs();
    if timer.elapsed < SUCCESS_DISPLAY_SECS {
        return;
    }
    transition_to_next_level(
        &mut commands,
        &mut terrain_res,
        body.0,
        &mut rng,
        &mut meshes,
        &mut materials,
        &terrain_entities,
        &ambient_entities,
        &mut ship_q,
    );
    next.set(AppState::GetReady);
    timer.transitioned = true;
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

/// Prep delay so meshes can upload, then terrain fade, “Get Ready” text, fade out, then play.
fn intro_tick(
    mut timer: ResMut<IntroTimer>,
    time: Res<Time>,
    mut next: ResMut<NextState<AppState>>,
    current_body: Res<CurrentBody>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    terrain_mat: Query<&MeshMaterial2d<ColorMaterial>, With<TerrainRoot>>,
    parallax_mat: Query<&MeshMaterial2d<ColorMaterial>, With<TerrainParallaxLayer>>,
    earth_water_mat: Query<&MeshMaterial2d<ColorMaterial>, With<EarthWaterFill>>,
    mut get_ready_bg: Query<&mut BackgroundColor, With<GetReadyRoot>>,
    mut get_ready_intro: ParamSet<(
        Query<(&mut Text, &mut TextColor), With<GetReadyBodyName>>,
        Query<&mut TextColor, With<GetReadyText>>,
    )>,
) {
    timer.elapsed += time.delta_secs();
    let t = timer.elapsed;

    // Timeline (seconds)
    const PREP: f32 = 0.14;
    const TERRAIN_FADE_DUR: f32 = 0.58;
    const TEXT_IN_START: f32 = 0.38;
    const TEXT_IN_END: f32 = 1.05;
    const HOLD_END: f32 = 1.75;
    const OUT_END: f32 = 2.55;

    // Terrain alpha 0 → 1 after prep
    let terrain_t0 = PREP;
    let terrain_t1 = PREP + TERRAIN_FADE_DUR;
    let terrain_a = if t <= terrain_t0 {
        0.0
    } else if t >= terrain_t1 {
        1.0
    } else {
        lerp(0.0, 1.0, (t - terrain_t0) / (terrain_t1 - terrain_t0))
    };

    let (tr, tg, tb) = current_body.0.terrain_surface_rgb();
    for mat_wrap in &terrain_mat {
        if let Some(mat) = materials.get_mut(mat_wrap.id()) {
            mat.color = Color::srgba(tr, tg, tb, terrain_a);
        }
    }
    for mat_wrap in &parallax_mat {
        if let Some(mat) = materials.get_mut(mat_wrap.id()) {
            mat.color = Color::srgba(tr, tg, tb, terrain_a * TERRAIN_PARALLAX_FAR_ALPHA);
        }
    }
    for mat_wrap in &earth_water_mat {
        if let Some(mat) = materials.get_mut(mat_wrap.id()) {
            let (wr, wg, wb) = EARTH_WATER_RGB;
            mat.color = Color::srgba(wr, wg, wb, terrain_a * EARTH_WATER_ALPHA);
        }
    }

    // Text + dim overlay
    let text_a = if t < TEXT_IN_START {
        0.0
    } else if t < TEXT_IN_END {
        lerp(0.0, 1.0, (t - TEXT_IN_START) / (TEXT_IN_END - TEXT_IN_START))
    } else if t < HOLD_END {
        1.0
    } else if t < OUT_END {
        lerp(1.0, 0.0, (t - HOLD_END) / (OUT_END - HOLD_END))
    } else {
        0.0
    };

    let overlay_a = if t < TEXT_IN_END {
        lerp(0.0, 0.42, ((t - TEXT_IN_START).max(0.0) / (TEXT_IN_END - TEXT_IN_START)).min(1.0))
    } else if t < HOLD_END {
        0.42
    } else if t < OUT_END {
        lerp(0.42, 0.0, (t - HOLD_END) / (OUT_END - HOLD_END))
    } else {
        0.0
    };

    let name = current_body.0.display_name();
    for (mut text, mut tc) in get_ready_intro.p0() {
        text.0.clear();
        text.0.push_str(name);
        **tc = Color::srgba(0.88, 0.93, 1.0, text_a);
    }
    for mut tc in get_ready_intro.p1() {
        **tc = Color::srgba(1.0, 0.94, 0.72, text_a);
    }
    for mut bg in &mut get_ready_bg {
        *bg = BackgroundColor(Color::srgba(0.0, 0.0, 0.0, overlay_a));
    }

    if t >= OUT_END {
        next.set(AppState::Playing);
    }
}

fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut terrain_res: ResMut<Terrain>,
    mut rng: ResMut<GameRng>,
    body: Res<CurrentBody>,
) {
    let pad_count = rng.0.gen_range(2..=3);
    *terrain_res = generate_terrain(&mut rng.0, pad_count, body.0);
    spawn_terrain_entity(
        &mut commands,
        &mut meshes,
        &mut materials,
        &terrain_res,
        body.0,
        &mut rng.0,
        true,
    );
    spawn_ambient_vfx(
        &mut commands,
        &terrain_res,
        body.0,
        &mut meshes,
        &mut materials,
    );
    crate::ship::spawn_ship(&mut commands, &mut meshes, &mut materials);
}

fn handle_ground_events(
    mut landed: MessageReader<LandedOnPad>,
    mut crashed: MessageReader<CrashEvent>,
    mut next: ResMut<NextState<AppState>>,
    mut body: ResMut<CurrentBody>,
    mut game_end: ResMut<GameEnd>,
    score: Res<Score>,
    mut high_scores: ResMut<HighScores>,
) {
    for _ in crashed.read() {
        game_end.reason = EndReason::Crashed;
        game_end.score = score.0;
        persistence::merge_and_persist(&mut high_scores.0, score.0);
        next.set(AppState::GameOver);
    }

    for ev in landed.read() {
        if body.0 == CelestialBody::Pluto {
            game_end.reason = EndReason::Victory;
            game_end.score = ev.total_score;
            persistence::merge_and_persist(&mut high_scores.0, ev.total_score);
            next.set(AppState::GameOver);
            continue;
        }

        if let Some(next_body) = body.0.next() {
            body.0 = next_body;
        }

        next.set(AppState::LandingSuccess);
    }
}

fn restart_input(
    state: Res<State<AppState>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next: ResMut<NextState<AppState>>,
    mut score: ResMut<Score>,
    mut high_scores: ResMut<HighScores>,
    mut body: ResMut<CurrentBody>,
    mut terrain_res: ResMut<Terrain>,
    mut rng: ResMut<GameRng>,
    mut commands: Commands,
    terrain_entities: Query<Entity, With<TerrainStack>>,
    ambient_entities: Query<Entity, With<crate::ambient_vfx::AmbientVfx>>,
    crash_fx: Query<Entity, With<CrashFx>>,
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
    high_scores.0 = persistence::load_high_scores();
    body.0 = CelestialBody::ORDER[0];

    despawn_ambient_vfx(&mut commands, &ambient_entities);
    for e in crash_fx.iter() {
        commands.entity(e).despawn();
    }
    for e in &terrain_entities {
        commands.entity(e).despawn();
    }

    let pad_count = rng.0.gen_range(2..=3);
    *terrain_res = generate_terrain(&mut rng.0, pad_count, body.0);
    spawn_terrain_entity(
        &mut commands,
        &mut meshes,
        &mut materials,
        &terrain_res,
        body.0,
        &mut rng.0,
        true,
    );
    spawn_ambient_vfx(
        &mut commands,
        &terrain_res,
        body.0,
        &mut meshes,
        &mut materials,
    );

    if ship_q.is_empty() {
        crate::ship::spawn_ship(&mut commands, &mut meshes, &mut materials);
    } else {
        for (mut ship, mut tf) in &mut ship_q {
            reset_ship_at_spawn(&mut ship, &mut tf);
        }
    }

    next.set(AppState::GetReady);
}
