//! Planet order and gravity scaling.

use crate::constants::{THRUST_DIAG, THRUST_MAIN};

/// Real surface gravity (m/s²) — used only for ratios; feel is tuned separately.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Planet {
    Earth,
    Moon,
    Mars,
    Mercury,
}

impl Planet {
    pub const ORDER: [Planet; 4] = [Planet::Earth, Planet::Moon, Planet::Mars, Planet::Mercury];

    pub fn real_gravity_m_s2(self) -> f32 {
        match self {
            Planet::Earth => 9.81,
            Planet::Moon => 1.62,
            Planet::Mars => 3.71,
            Planet::Mercury => 3.7,
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Planet::Earth => "Earth (training)",
            Planet::Moon => "Moon",
            Planet::Mars => "Mars",
            Planet::Mercury => "Mercury",
        }
    }

    pub fn next(self) -> Option<Planet> {
        match self {
            Planet::Earth => Some(Planet::Moon),
            Planet::Moon => Some(Planet::Mars),
            Planet::Mars => Some(Planet::Mercury),
            Planet::Mercury => None,
        }
    }
}

/// Arcade gravity in world units/s² (scaled from real ratios vs Earth).
pub fn gravity_acceleration(planet: Planet) -> f32 {
    const EARTH_G: f32 = 9.81;
    const BASE: f32 = 320.0;
    BASE * planet.real_gravity_m_s2() / EARTH_G
}

/// Thruster strengths stay consistent; gravity changes by planet.
pub fn thrust_main() -> f32 {
    THRUST_MAIN
}

pub fn thrust_diag() -> f32 {
    THRUST_DIAG
}
