//! HUD: fuel, velocity, planet; game over overlay.

use bevy::prelude::*;

use crate::game_flow::{AppState, CurrentPlanet, EndReason, GameEnd};
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
        app.add_systems(Startup, spawn_hud).add_systems(
            Update,
            (
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
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                right: Val::Px(12.0),
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
                Text::new("Planet: Earth"),
                TextFont {
                    font_size: 18.0,
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
                Text::new("vx: 0.0  vy: 0.0"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.85, 0.9, 1.0)),
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

fn update_hud(
    planet: Res<CurrentPlanet>,
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
        &mut Text,
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

    for mut t in &mut planet_q {
        t.0 = format!("Planet: {}", planet.0.display_name());
    }

    for mut t in &mut fuel_q {
        t.0 = format!("Methane: {:.0}%", ship.fuel);
    }

    for mut t in &mut vel_q {
        t.0 = format!("vx: {:.1}  vy: {:.1}", ship.velocity.x, ship.velocity.y);
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
                    "You landed on Mercury!\nFinal score: {}\nPress R to restart",
                    game_end.score
                )
            }
        };
    } else {
        *vis = Visibility::Hidden;
        text.0.clear();
    }
}
