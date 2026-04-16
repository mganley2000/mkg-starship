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

/// Methane units per second while any thruster is active.
pub const FUEL_BURN_RATE: f32 = 28.0;
pub const INITIAL_FUEL: f32 = 200.0;

/// Maximum safe landing speed (world units / s).
pub const SAFE_LANDING_VY: f32 = 220.0;
pub const SAFE_LANDING_VX: f32 = 120.0;

/// Base score per successful landing.
pub const SCORE_BASE_LANDING: i32 = 500;

/// Terrain mesh extends down to this y (must stay below the lowest surface point).
pub const TERRAIN_FLOOR_Y: f32 = -WORLD_HEIGHT * 0.78;

/// Number of terrain samples along x (more samples → sharper local corners).
pub const TERRAIN_SAMPLES: usize = 140;

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
