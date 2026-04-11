//! Ground contact: safe landing vs crash, emits events.

use bevy::prelude::*;

use crate::constants::{SAFE_LANDING_VX, SAFE_LANDING_VY, SCORE_BASE_LANDING, SHIP_HALF_HEIGHT};
use crate::game_flow::{CrashEvent, LandedOnPad, Score};
use crate::ship::{Ship, ship_foot_x, ship_foot_y};
use crate::terrain::{Terrain, is_on_pad, terrain_height_at};

pub fn ground_contact(
    terrain: Res<Terrain>,
    mut ship_q: Query<(&mut Ship, &mut Transform), With<crate::ship::ShipRoot>>,
    mut landed: MessageWriter<LandedOnPad>,
    mut crashed: MessageWriter<CrashEvent>,
    mut score: ResMut<Score>,
) {
    let tolerance = 14.0_f32;

    for (mut ship, mut tf) in &mut ship_q {
        let x = ship_foot_x(&tf);
        let foot = ship_foot_y(&tf);
        let h = terrain_height_at(&terrain, x);

        let crossing = foot <= h + 2.0 && ship.foot_prev > h + 1.0;
        if !crossing {
            continue;
        }

        tf.translation.y = h + SHIP_HALF_HEIGHT + 0.5;
        let foot = ship_foot_y(&tf);

        let vy = ship.velocity.y;
        let vx = ship.velocity.x;

        if let Some(_pad) = is_on_pad(&terrain, x, foot, tolerance) {
            if vy.abs() <= SAFE_LANDING_VY && vx.abs() <= SAFE_LANDING_VX {
                let bonus = ((SAFE_LANDING_VY - vy.abs()) * 0.8
                    + (SAFE_LANDING_VX - vx.abs()) * 0.3)
                    .max(0.0) as i32;
                score.0 += SCORE_BASE_LANDING + bonus;
                ship.velocity = Vec2::ZERO;
                landed.write(LandedOnPad {
                    total_score: score.0,
                });
            } else {
                crashed.write(CrashEvent);
            }
        } else {
            crashed.write(CrashEvent);
        }
    }
}
