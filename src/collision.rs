//! Ground contact: safe landing vs crash, emits events.

use bevy::prelude::*;

use crate::constants::{
    compute_landing_score, LEVEL_GROUND_TOLERANCE, SAFE_LANDING_VX, SAFE_LANDING_VY,
    SHIP_FOOT_OFFSET_Y, SHIP_HULL_HALF_WIDTH, TERRAIN_FLOOR_Y, TERRAIN_TUNNEL_CRASH_DEPTH,
    WORLD_VIEW_BOTTOM_Y,
};
use crate::game_flow::{CrashEvent, CurrentBody, LandedOnPad, LevelFlightTimer, Score};
use crate::planets::gravity_acceleration;
use crate::ship::{Ship, ship_foot_y, ship_hull_bottom_y, ship_left_foot_x, ship_right_foot_x};
use crate::terrain::{Terrain, is_on_pad, terrain_height_at, terrain_max_height_in_span};

fn same_landing_pad(
    terrain: &Terrain,
    x_left: f32,
    x_right: f32,
    foot_y: f32,
    tolerance: f32,
) -> bool {
    match (
        is_on_pad(terrain, x_left, foot_y, tolerance),
        is_on_pad(terrain, x_right, foot_y, tolerance),
    ) {
        (Some(a), Some(b)) => {
            (a.x_min - b.x_min).abs() < 0.5
                && (a.x_max - b.x_max).abs() < 0.5
                && (a.y_top - b.y_top).abs() < 0.5
        }
        _ => false,
    }
}

pub fn ground_contact(
    terrain: Res<Terrain>,
    mut ship_q: Query<(&mut Ship, &mut Transform), With<crate::ship::ShipRoot>>,
    mut landed: MessageWriter<LandedOnPad>,
    mut crashed: MessageWriter<CrashEvent>,
    mut score: ResMut<Score>,
    level_timer: Res<LevelFlightTimer>,
    current_body: Res<CurrentBody>,
) {
    let tolerance = 14.0_f32;

    for (mut ship, mut tf) in &mut ship_q {
        let cx = tf.translation.x;
        let xl = ship_left_foot_x(&tf);
        let xr = ship_right_foot_x(&tf);
        let foot = ship_foot_y(&tf);
        let hull_bottom = ship_hull_bottom_y(&tf);

        let h_l = terrain_height_at(&terrain, xl);
        let h_r = terrain_height_at(&terrain, xr);
        let ground_ref = h_l.min(h_r);

        // Highest terrain along the hull bottom span (hull hits this before feet).
        let h_max_span = terrain_max_height_in_span(&terrain, cx, SHIP_HULL_HALF_WIDTH);

        // Fell past the bottom of the view or through the terrain mesh → always crash.
        if foot < WORLD_VIEW_BOTTOM_Y || foot < TERRAIN_FLOOR_Y + 60.0 {
            crashed.write(CrashEvent {
                origin: Vec2::new(cx, foot),
            });
            continue;
        }

        // Tunneling: feet deep below foot-line surface, or hull deep below a peak under the ship.
        if foot < ground_ref - TERRAIN_TUNNEL_CRASH_DEPTH
            || hull_bottom < h_max_span - TERRAIN_TUNNEL_CRASH_DEPTH
        {
            crashed.write(CrashEvent {
                origin: Vec2::new(cx, foot),
            });
            continue;
        }

        let fast_down = ship.velocity.y < -280.0;

        // Foot line: landing contact at the legs.
        let narrow_foot = foot <= ground_ref + 2.0 && ship.foot_prev > ground_ref + 1.0;
        let broad_foot = foot <= ground_ref + 4.0
            && ship.foot_prev > ground_ref - 1.0
            && fast_down;

        // Hull bottom edge: hits terrain first on peaks between or under the hull.
        let narrow_hull =
            hull_bottom <= h_max_span + 2.0 && ship.hull_bottom_prev > h_max_span + 1.0;
        let broad_hull = hull_bottom <= h_max_span + 4.0
            && ship.hull_bottom_prev > h_max_span - 1.0
            && fast_down;

        let crossing = narrow_foot || narrow_hull || broad_foot || broad_hull;

        if !crossing {
            continue;
        }

        tf.translation.y = ground_ref - SHIP_FOOT_OFFSET_Y + 0.5;
        let foot = ship_foot_y(&tf);
        let h_l = terrain_height_at(&terrain, xl);
        let h_r = terrain_height_at(&terrain, xr);

        let vy = ship.velocity.y;
        let vx = ship.velocity.x;

        let level = (h_l - h_r).abs() <= LEVEL_GROUND_TOLERANCE;
        let on_pad = same_landing_pad(&terrain, xl, xr, foot, tolerance);

        if on_pad && level {
            if vy.abs() <= SAFE_LANDING_VY && vx.abs() <= SAFE_LANDING_VX {
                let g = gravity_acceleration(current_body.0);
                let pts = compute_landing_score(level_timer.elapsed, ship.fuel, g);
                score.0 += pts;
                ship.velocity = Vec2::ZERO;
                landed.write(LandedOnPad {
                    total_score: score.0,
                });
            } else {
                crashed.write(CrashEvent {
                    origin: Vec2::new(cx, foot),
                });
            }
        } else {
            crashed.write(CrashEvent {
                origin: Vec2::new(cx, foot),
            });
        }
    }
}
