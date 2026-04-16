//! Gravity, thrusters, integration. Runs in `Update` while playing (same clock as input).

use bevy::prelude::*;

use crate::constants::{
    FUEL_BURN_RATE, MAX_PHYSICS_DT, SHIP_FOOT_OFFSET_Y, SHIP_HULL_BOTTOM_OFFSET_Y, WORLD_HEIGHT,
    WORLD_WIDTH, thrust_side_components,
};
use crate::planets::{gravity_acceleration, thrust_main};
use crate::ship::{Ship, ShipRoot};

pub fn ship_physics(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut q: Query<(&mut Ship, &mut Transform), With<ShipRoot>>,
    current: Res<crate::game_flow::CurrentBody>,
) {
    let dt = time.delta_secs().clamp(0.0, MAX_PHYSICS_DT);
    let g = gravity_acceleration(current.0);
    let gravity = Vec2::new(0.0, -g);

    let main = thrust_main();
    let (h_comp, v_comp) = thrust_side_components();

    for (mut ship, mut tf) in &mut q {
        ship.foot_prev = tf.translation.y + SHIP_FOOT_OFFSET_Y;
        ship.hull_bottom_prev = tf.translation.y + SHIP_HULL_BOTTOM_OFFSET_Y;

        let mut accel = gravity;

        let mut burning = false;
        if ship.fuel > 0.0 {
            if keyboard.pressed(KeyCode::ArrowDown) {
                accel += Vec2::new(0.0, main);
                burning = true;
            }
            // Left-side thruster (←): push ship right + up (counter leftward drift).
            if keyboard.pressed(KeyCode::ArrowLeft) {
                accel += Vec2::new(h_comp, v_comp);
                burning = true;
            }
            // Right-side thruster (→): push ship left + up (counter rightward drift).
            if keyboard.pressed(KeyCode::ArrowRight) {
                accel += Vec2::new(-h_comp, v_comp);
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
