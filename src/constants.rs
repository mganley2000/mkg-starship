//! Tunable gameplay constants (world units ≈ pixels with default camera).

/// Playfield width (world x from -HALF to +HALF).
pub const WORLD_WIDTH: f32 = 1200.0;
/// Playfield height used for layout and camera.
pub const WORLD_HEIGHT: f32 = 800.0;

pub const SHIP_HALF_WIDTH: f32 = 22.0;
pub const SHIP_HALF_HEIGHT: f32 = 28.0;

/// Starting height for the ship (center y).
pub fn ship_spawn_y() -> f32 {
    WORLD_HEIGHT * 0.38
}

/// Terrain mesh extends down to this y.
pub const TERRAIN_FLOOR_Y: f32 = -WORLD_HEIGHT * 0.48;

/// Number of terrain samples along x.
pub const TERRAIN_SAMPLES: usize = 96;

/// Main engine thrust (world units / s²).
pub const THRUST_MAIN: f32 = 520.0;
/// Diagonal thrusters (45° from horizontal).
pub const THRUST_DIAG: f32 = 360.0;

/// Methane units per second while any thruster is active.
pub const FUEL_BURN_RATE: f32 = 28.0;
pub const INITIAL_FUEL: f32 = 100.0;

/// Maximum safe landing speed (world units / s).
pub const SAFE_LANDING_VY: f32 = 220.0;
pub const SAFE_LANDING_VX: f32 = 120.0;

/// Base score per successful landing.
pub const SCORE_BASE_LANDING: i32 = 500;
