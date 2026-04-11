//! Gravity, thrusters, integration. Runs in `FixedUpdate` while playing.

use bevy::prelude::*;

use crate::constants::{FUEL_BURN_RATE, SHIP_HALF_HEIGHT, WORLD_HEIGHT, WORLD_WIDTH};
use crate::planets::{gravity_acceleration, thrust_diag, thrust_main};
use crate::ship::{Ship, ShipRoot};

pub fn ship_physics(
    time: Res<Time<Fixed>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut q: Query<(&mut Ship, &mut Transform), With<ShipRoot>>,
    current: Res<crate::game_flow::CurrentPlanet>,
) {
    let dt = time.delta_secs();
    let g = gravity_acceleration(current.0);
    let gravity = Vec2::new(0.0, -g);

    let main = thrust_main();
    let diag = thrust_diag();
    let d_diag = Vec2::new(1.0, 1.0).normalize() * diag;
    let d_diag_r = Vec2::new(-1.0, 1.0).normalize() * diag;

    for (mut ship, mut tf) in &mut q {
        ship.foot_prev = tf.translation.y - SHIP_HALF_HEIGHT;

        let mut accel = gravity;

        let mut burning = false;
        if ship.fuel > 0.0 {
            if keyboard.pressed(KeyCode::ArrowDown) {
                accel += Vec2::new(0.0, main);
                burning = true;
            }
            if keyboard.pressed(KeyCode::KeyZ) {
                accel += d_diag;
                burning = true;
            }
            if keyboard.pressed(KeyCode::Slash) {
                accel += d_diag_r;
                burning = true;
            }
            if burning {
                ship.fuel = (ship.fuel - FUEL_BURN_RATE * dt).max(0.0);
            }
        }

        ship.velocity += accel * dt;
        tf.translation += (ship.velocity * dt).extend(0.0);

        let half_w = WORLD_WIDTH * 0.5;
        let half_h = WORLD_HEIGHT * 0.5;
        if tf.translation.x < -half_w {
            tf.translation.x = -half_w;
            ship.velocity.x *= -0.25;
        } else if tf.translation.x > half_w {
            tf.translation.x = half_w;
            ship.velocity.x *= -0.25;
        }
        if tf.translation.y > half_h {
            tf.translation.y = half_h;
            ship.velocity.y = ship.velocity.y.min(0.0);
        }
    }
}
