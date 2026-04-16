//! Tunable gameplay constants (world units ≈ pixels with default camera).

/// Playfield width (world x from -HALF to +HALF).
pub const WORLD_WIDTH: f32 = 1200.0;
/// Playfield height used for layout and camera.
pub const WORLD_HEIGHT: f32 = 800.0;
/// Bottom edge of the orthographic view (world y); ship feet below this count as lost.
pub const WORLD_VIEW_BOTTOM_Y: f32 = -WORLD_HEIGHT * 0.5;
/// Feet this far below the local terrain surface = tunneling / missed collision → crash.
pub const TERRAIN_TUNNEL_CRASH_DEPTH: f32 = 12.0;

/// Half horizontal distance from ship center to each landing foot (terrain samples here).
pub const FOOT_SEP_X: f32 = 18.0;
/// Foot line offset from ship center (feet are below center; foot_y = center.y + this).
pub const SHIP_FOOT_OFFSET_Y: f32 = -26.0;
/// Hull triangle base (bottom edge) y offset from ship center — must match `hull_triangle_mesh` in `ship.rs`.
pub const SHIP_HULL_BOTTOM_OFFSET_Y: f32 = -15.0;
/// Half-width along x for sampling terrain under the hull bottom (world units).
pub const SHIP_HULL_HALF_WIDTH: f32 = 20.0;

/// Max difference in terrain height under the two feet for a valid "level" landing.
pub const LEVEL_GROUND_TOLERANCE: f32 = 5.0;

/// Main engine thrust (world units / s²).
pub const THRUST_MAIN: f32 = 520.0;
/// Side thrusters: total magnitude budget; split 50% horizontal / 50% vertical (up).
pub const THRUST_SIDE: f32 = 360.0;

/// Max delta time (seconds) for one physics integration step — avoids huge jumps after a hitch.
pub const MAX_PHYSICS_DT: f32 = 0.05;

/// Height above terrain (SI) within which landing dust is spawned (see `landing_dust`).
pub const LANDING_DUST_ALTITUDE_METERS: f32 = 1.0;

/// Methane units per second while any thruster is active.
pub const FUEL_BURN_RATE: f32 = 28.0;
pub const INITIAL_FUEL: f32 = 200.0;

/// Maximum safe landing speed (world units / s).
pub const SAFE_LANDING_VY: f32 = 220.0;
pub const SAFE_LANDING_VX: f32 = 120.0;

/// Landing score: faster times (lower seconds) yield more points: `NUMERATOR / (t + FLOOR)`.
pub const SCORE_TIME_NUMERATOR: f32 = 900.0;
pub const SCORE_TIME_FLOOR_SEC: f32 = 0.35;
/// Landing score: fuel fraction (0–1) × this value.
pub const SCORE_FUEL_MULTIPLIER: f32 = 480.0;
/// Bonus per landing: `round(10 × gravity_acceleration(body))` (see `planets::gravity_acceleration`).
pub const SCORE_GRAVITY_BONUS_MUL: f32 = 10.0;

/// Points for one successful landing: faster elapsed time and more fuel yield more; plus gravity bonus.
#[inline]
pub fn compute_landing_score(land_time_sec: f32, fuel_remaining: f32, gravity_acceleration: f32) -> i32 {
    let t = land_time_sec.max(0.01);
    let time_pts = (SCORE_TIME_NUMERATOR / (t + SCORE_TIME_FLOOR_SEC)) as i32;
    let fuel_frac = (fuel_remaining / INITIAL_FUEL).clamp(0.0, 1.0);
    let fuel_pts = (fuel_frac * SCORE_FUEL_MULTIPLIER) as i32;
    let grav_pts = (SCORE_GRAVITY_BONUS_MUL * gravity_acceleration + 0.5).floor() as i32;
    time_pts + fuel_pts + grav_pts
}

/// Terrain mesh extends down to this y (must stay below the lowest surface point).
pub const TERRAIN_FLOOR_Y: f32 = -WORLD_HEIGHT * 0.78;

/// Number of terrain samples along x (more samples → sharper local corners).
pub const TERRAIN_SAMPLES: usize = 140;

/// Earth: water table height = min(surface y) + this × (max − min) of the heightfield; valleys below fill with water.
pub const EARTH_WATER_TABLE_FRAC: f32 = 0.40;
pub const EARTH_WATER_RGB: (f32, f32, f32) = (0.12, 0.42, 0.78);
pub const EARTH_WATER_ALPHA: f32 = 0.88;

/// Single background parallax: random heightfield (no pads). World Y offset, z depth, alpha × terrain tint.
/// Smoothing pass count for parallax = `ceil(primary_smoothing_passes × this)` (50% more passes → ~50% smoother).
pub const TERRAIN_PARALLAX_SMOOTHING_MULT: f32 = 1.5;
/// Multiplies the sine layer frequencies (11, 27, 53, 91) for parallax only — lower = fewer peaks across the width.
pub const TERRAIN_PARALLAX_SIN_FREQ_SCALE: f32 = 0.40;
/// Scales per-sample noise amplitude for parallax (main terrain uses full `noise_half`).
pub const TERRAIN_PARALLAX_NOISE_SCALE: f32 = 0.52;
/// Scales micro-spike probability and amplitude for parallax vs main terrain profile.
pub const TERRAIN_PARALLAX_MICRO_PROB_SCALE: f32 = 0.28;
pub const TERRAIN_PARALLAX_MICRO_AMP_SCALE: f32 = 0.55;

pub const TERRAIN_PARALLAX_FAR_Y: f32 = 275.0;
/// Parallax mesh fill bottom in **local** space. Must be ≤ `WORLD_VIEW_BOTTOM_Y - TERRAIN_PARALLAX_FAR_Y` so the layer still covers the viewport bottom after the parallax transform (otherwise a horizontal seam appears above the screen edge).
pub const TERRAIN_PARALLAX_MESH_FLOOR_Y: f32 = WORLD_VIEW_BOTTOM_Y - TERRAIN_PARALLAX_FAR_Y;
pub const TERRAIN_PARALLAX_FAR_Z: f32 = -0.26;
pub const TERRAIN_PARALLAX_FAR_ALPHA: f32 = 0.032;

/// Starting height for the ship (center y).
pub fn ship_spawn_y() -> f32 {
    WORLD_HEIGHT * 0.38
}

/// Side thruster horizontal and vertical components (each 50% of `THRUST_SIDE`).
#[inline]
pub fn thrust_side_components() -> (f32, f32) {
    let h = THRUST_SIDE * 0.5;
    (h, h)
}
