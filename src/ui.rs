//! HUD: fuel, velocity, planet; game over overlay.

use bevy::prelude::*;

use crate::constants::{INITIAL_FUEL, SAFE_LANDING_VY};
use crate::planets::game_velocity_to_m_s;
use crate::game_flow::{
    AppState, CurrentBody, EndReason, GameEnd, GetReadyRoot, GetReadyText, SuccessRoot,
    SuccessText,
};
use crate::ship::Ship;
use crate::ship::ShipRoot;

#[derive(Component)]
struct HudRoot;

#[derive(Component)]
struct HudFuel;

#[derive(Component)]
struct HudVel;

#[derive(Component)]
struct HudPlanet;

#[derive(Component)]
struct HudGameOver;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_hud, spawn_intro_overlay, spawn_success_overlay))
            .add_systems(
                Update,
                (
                    sync_hud_and_intro_visibility,
                    update_hud.run_if(in_state(AppState::Playing)),
                    update_game_over_overlay,
                ),
            );
    }
}

fn spawn_hud(mut commands: Commands) {
    commands
        .spawn((
            HudRoot,
            Visibility::Hidden,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                right: Val::Px(12.0),
                width: Val::Px(300.0),
                min_height: Val::Px(108.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexEnd,
                row_gap: Val::Px(6.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
        ))
        .with_children(|p| {
            p.spawn((
                HudPlanet,
                Text::new("Mercury (3.70 m/s^2)"),
                TextFont {
                    font_size: 17.0,
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.95, 1.0)),
            ));
            p.spawn((
                HudFuel,
                Text::new("Methane: 100%"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 1.0, 0.75)),
            ));
            p.spawn((
                HudVel,
                Text::new("Vy:   +0.0 m/s"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.95, 0.95)),
            ));
        });

    commands.spawn((
        HudGameOver,
        Text::new(""),
        TextFont {
            font_size: 28.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.85, 0.3)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(50.0),
            top: Val::Percent(42.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        Visibility::Hidden,
    ));
}

fn spawn_intro_overlay(mut commands: Commands) {
    commands
        .spawn((
            GetReadyRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
            Visibility::Visible,
        ))
        .with_children(|p| {
            p.spawn((
                GetReadyText,
                Text::new("Get Ready"),
                TextFont {
                    font_size: 52.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 0.94, 0.72, 0.0)),
            ));
        });
}

fn spawn_success_overlay(mut commands: Commands) {
    commands
        .spawn((
            SuccessRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
            Visibility::Hidden,
        ))
        .with_children(|p| {
            p.spawn((
                SuccessText,
                Text::new("Success"),
                TextFont {
                    font_size: 52.0,
                    ..default()
                },
                TextColor(Color::srgb(0.55, 1.0, 0.65)),
            ));
        });
}

fn sync_hud_and_intro_visibility(
    state: Res<State<AppState>>,
    mut q: ParamSet<(
        Query<&mut Visibility, With<HudRoot>>,
        Query<&mut Visibility, With<GetReadyRoot>>,
        Query<&mut Visibility, With<SuccessRoot>>,
    )>,
) {
    let playing = *state.get() == AppState::Playing;
    let get_ready = *state.get() == AppState::GetReady;
    let landing_success = *state.get() == AppState::LandingSuccess;

    for mut v in q.p0() {
        *v = if playing {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for mut v in q.p1() {
        *v = if get_ready {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for mut v in q.p2() {
        *v = if landing_success {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn update_hud(
    body: Res<CurrentBody>,
    ship: Query<
        &Ship,
        (
            With<ShipRoot>,
            Without<HudPlanet>,
            Without<HudFuel>,
            Without<HudVel>,
            Without<HudGameOver>,
        ),
    >,
    mut fuel_q: Query<
        &mut Text,
        (With<HudFuel>, Without<HudVel>, Without<HudPlanet>),
    >,
    mut vel_q: Query<
        (&mut Text, &mut TextColor),
        (With<HudVel>, Without<HudFuel>, Without<HudPlanet>),
    >,
    mut planet_q: Query<
        &mut Text,
        (With<HudPlanet>, Without<HudFuel>, Without<HudVel>),
    >,
) {
    let Ok(ship) = ship.single() else {
        return;
    };

    let g = body.0.real_gravity_m_s2();
    for mut t in &mut planet_q {
        t.0 = format!("{} ({:.2} m/s^2)", body.0.display_name(), g);
    }

    for mut t in &mut fuel_q {
        t.0 = format!(
            "Methane: {:>3.0}%",
            (ship.fuel / INITIAL_FUEL * 100.0).clamp(0.0, 100.0)
        );
    }

    let vy = ship.velocity.y;
    let vy_ms = game_velocity_to_m_s(vy);
    let vy_abs = vy.abs();
    let vx_ms = game_velocity_to_m_s(ship.velocity.x);
    let safe = SAFE_LANDING_VY;
    let warn_lo = safe * 0.8;

    // Horizontal speed (m/s) above which landing is unsafe — Vy line turns purple.
    const MAX_LANDING_HORIZ_M_S: f32 = 5.0;

    let vel_color = if vx_ms.abs() > MAX_LANDING_HORIZ_M_S {
        // Too much sideways drift for a safe landing (vertical color shows this state).
        Color::srgb(0.72, 0.42, 0.95)
    } else if vy_abs > safe {
        Color::srgb(0.95, 0.25, 0.2)
    } else if vy_abs > warn_lo {
        Color::srgb(0.95, 0.95, 0.95)
    } else {
        Color::srgb(0.35, 0.92, 0.45)
    };

    for (mut t, mut tc) in &mut vel_q {
        t.0 = format!("Vy: {:+7.1} m/s", vy_ms);
        **tc = vel_color;
    }
}

fn update_game_over_overlay(
    state: Res<State<AppState>>,
    game_end: Res<GameEnd>,
    mut q: Query<(&mut Text, &mut Visibility), With<HudGameOver>>,
) {
    let Ok((mut text, mut vis)) = q.single_mut() else {
        return;
    };

    if *state.get() == AppState::GameOver {
        *vis = Visibility::Visible;
        text.0 = match game_end.reason {
            EndReason::Crashed => {
                format!("Game Over\nScore: {}\nPress R to restart", game_end.score)
            }
            EndReason::Victory => {
                format!(
                    "Solar system complete!\nFinal score: {}\nPress R to restart",
                    game_end.score
                )
            }
        };
    } else {
        *vis = Visibility::Hidden;
        text.0.clear();
    }
}
