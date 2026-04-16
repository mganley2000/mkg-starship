//! Celestial bodies: planets and major moons — order and gravity scaling.

use crate::constants::THRUST_MAIN;

/// Surfaces to land on (planets + selected moons), in game progression order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CelestialBody {
    Mercury,
    Venus,
    Earth,
    Moon,
    Mars,
    Io,
    Europa,
    Ganymede,
    Callisto,
    Jupiter,
    Saturn,
    Enceladus,
    Titan,
    Uranus,
    Neptune,
    Pluto,
}

impl CelestialBody {
    pub const ORDER: [CelestialBody; 16] = [
        CelestialBody::Mercury,
        CelestialBody::Venus,
        CelestialBody::Earth,
        CelestialBody::Moon,
        CelestialBody::Mars,
        CelestialBody::Io,
        CelestialBody::Europa,
        CelestialBody::Ganymede,
        CelestialBody::Callisto,
        CelestialBody::Saturn,
        CelestialBody::Enceladus,
        CelestialBody::Titan,
        CelestialBody::Uranus,
        CelestialBody::Neptune,
        CelestialBody::Pluto,
        CelestialBody::Jupiter,
    ];

    pub fn real_gravity_m_s2(self) -> f32 {
        match self {
            // Planets — surface or 1-bar reference (m/s²).
            CelestialBody::Mercury => 3.7,
            CelestialBody::Venus => 8.87,
            CelestialBody::Earth => 9.81,
            CelestialBody::Moon => 1.62,
            CelestialBody::Mars => 3.71,
            CelestialBody::Jupiter => 24.79,
            CelestialBody::Saturn => 10.44,
            CelestialBody::Uranus => 8.69,
            CelestialBody::Neptune => 11.15,
            CelestialBody::Pluto => 0.62,
            // Moons — approximate surface gravity (m/s²).
            CelestialBody::Io => 1.796,
            CelestialBody::Europa => 1.314,
            CelestialBody::Ganymede => 1.428,
            CelestialBody::Callisto => 1.235,
            CelestialBody::Enceladus => 0.113,
            CelestialBody::Titan => 1.352,
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            CelestialBody::Mercury => "Mercury",
            CelestialBody::Venus => "Venus",
            CelestialBody::Earth => "Earth",
            CelestialBody::Moon => "Moon",
            CelestialBody::Mars => "Mars",
            CelestialBody::Jupiter => "Jupiter",
            CelestialBody::Saturn => "Saturn",
            CelestialBody::Uranus => "Uranus",
            CelestialBody::Neptune => "Neptune",
            CelestialBody::Pluto => "Pluto",
            CelestialBody::Io => "Io",
            CelestialBody::Europa => "Europa",
            CelestialBody::Ganymede => "Ganymede",
            CelestialBody::Callisto => "Callisto",
            CelestialBody::Enceladus => "Enceladus",
            CelestialBody::Titan => "Titan",
        }
    }

    /// RGB tint (0–1) for the terrain mesh; alpha is applied separately for fades.
    pub fn terrain_surface_rgb(self) -> (f32, f32, f32) {
        match self {
            CelestialBody::Mercury => (0.78, 0.66, 0.54), // light brown, cratered rock
            // Light purple + light blue (blended — single fill color)
            CelestialBody::Venus => (0.72, 0.62, 0.88),
            CelestialBody::Earth => (0.22, 0.58, 0.32), // green land
            CelestialBody::Moon => (0.52, 0.52, 0.55),   // regolith / rayed craters
            CelestialBody::Mars => (0.62, 0.34, 0.22),  // rusty regolith
            CelestialBody::Jupiter => (0.58, 0.48, 0.38), // banded clouds / tan
            CelestialBody::Io => (0.82, 0.72, 0.38),     // sulfur yellow-orange
            CelestialBody::Europa => (0.86, 0.90, 0.96), // water ice
            CelestialBody::Ganymede => (0.54, 0.52, 0.50), // mixed ice / rock
            CelestialBody::Callisto => (0.44, 0.42, 0.40), // dark cratered ice
            CelestialBody::Saturn => (0.76, 0.70, 0.56), // pale banded
            CelestialBody::Enceladus => (0.90, 0.93, 0.98), // bright ice
            CelestialBody::Titan => (0.52, 0.42, 0.30),  // organic haze / orange-brown
            CelestialBody::Uranus => (0.48, 0.78, 0.86), // cyan atmosphere
            CelestialBody::Neptune => (0.28, 0.42, 0.82), // deep blue
            CelestialBody::Pluto => (0.62, 0.52, 0.55),  // pink-brown nitrogen ice
        }
    }

    pub fn next(self) -> Option<CelestialBody> {
        match self {
            CelestialBody::Mercury => Some(CelestialBody::Venus),
            CelestialBody::Venus => Some(CelestialBody::Earth),
            CelestialBody::Earth => Some(CelestialBody::Moon),
            CelestialBody::Moon => Some(CelestialBody::Mars),
            CelestialBody::Mars => Some(CelestialBody::Io),
            CelestialBody::Io => Some(CelestialBody::Europa),
            CelestialBody::Europa => Some(CelestialBody::Ganymede),
            CelestialBody::Ganymede => Some(CelestialBody::Callisto),
            CelestialBody::Callisto => Some(CelestialBody::Jupiter),
            CelestialBody::Jupiter => Some(CelestialBody::Saturn),
            CelestialBody::Saturn => Some(CelestialBody::Enceladus),
            CelestialBody::Enceladus => Some(CelestialBody::Titan),
            CelestialBody::Titan => Some(CelestialBody::Uranus),
            CelestialBody::Uranus => Some(CelestialBody::Neptune),
            CelestialBody::Neptune => Some(CelestialBody::Pluto),
            CelestialBody::Pluto => None,
        }
    }
}

/// Arcade gravity in world units/s² (scaled from real ratios vs Earth).
pub fn gravity_acceleration(body: CelestialBody) -> f32 {
    const EARTH_G: f32 = 9.81;
    const BASE: f32 = 320.0;
    BASE * body.real_gravity_m_s2() / EARTH_G
}

/// Convert arcade vertical velocity (world units/s) to SI m/s (same scale as `gravity_acceleration`).
#[inline]
pub fn game_velocity_to_m_s(vy_game: f32) -> f32 {
    const EARTH_G: f32 = 9.81;
    const BASE: f32 = 320.0;
    vy_game * EARTH_G / BASE
}

/// Vertical distance in game units → SI meters (same scale as [`game_velocity_to_m_s`]).
#[inline]
pub fn game_vertical_distance_to_meters(d_game: f32) -> f32 {
    const EARTH_G: f32 = 9.81;
    const BASE: f32 = 320.0;
    d_game * EARTH_G / BASE
}

pub fn thrust_main() -> f32 {
    THRUST_MAIN
}

/// Methane baseline × this = actual thrust (Jupiter plasma is 4× stronger).
#[inline]
pub fn thrust_multiplier(body: CelestialBody) -> f32 {
    match body {
        CelestialBody::Jupiter => 2.0,
        _ => 1.0,
    }
}

/// HUD label for the current propellant type.
#[inline]
pub fn fuel_display_name(body: CelestialBody) -> &'static str {
    match body {
        CelestialBody::Jupiter => "Plasma",
        _ => "Methane",
    }
}
