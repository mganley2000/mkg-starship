//! HUD: fuel, velocity, planet; game over overlay.

use bevy::prelude::*;
use bevy::text::prelude::{Justify, TextLayout};

use crate::constants::{INITIAL_FUEL, SAFE_LANDING_VY};
use crate::planets::{CelestialBody, fuel_display_name, game_velocity_to_m_s};
use crate::game_flow::{
    AppState, CurrentBody, EndReason, GameEnd, GetReadyBodyName, GetReadyRoot, GetReadyText,
    HighScores, LevelFlightTimer, Score as RunScore, SuccessRoot, SuccessText,
};
use crate::persistence::format_high_score_timestamp;
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
struct HudTimer;

#[derive(Component)]
struct HudRunScore;

#[derive(Component)]
struct HudGameOverRoot;

#[derive(Component)]
struct HudGameOver;

#[derive(Component)]
struct TopScoresRoot;

#[derive(Component)]
struct TopScoresText;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
                Startup,
                (
                    spawn_hud,
                    spawn_intro_overlay,
                    spawn_success_overlay,
                    spawn_top_scores_bar,
                ),
            )
            .add_systems(
                Update,
                (
                    sync_hud_and_intro_visibility,
                    update_hud.run_if(in_state(AppState::Playing)),
                    update_top_scores_text,
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
                min_height: Val::Px(132.0),
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
                HudTimer,
                Text::new("Time: 0.0 s"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.85, 0.92, 1.0)),
            ));
            p.spawn((
                HudRunScore,
                Text::new("Run: 0"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.92, 0.55)),
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
                Text::new("  +0.0 m/s"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.95, 0.95)),
            ));
        });

    commands
        .spawn((
            HudGameOverRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            Visibility::Hidden,
        ))
        .with_children(|p| {
            p.spawn((
                HudGameOver,
                Text::new(""),
                TextLayout::new_with_justify(Justify::Center),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.85, 0.3)),
            ));
        });
}

fn spawn_top_scores_bar(mut commands: Commands) {
    commands
        .spawn((
            TopScoresRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                bottom: Val::Px(12.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(16.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
            Visibility::Visible,
        ))
        .with_children(|p| {
            p.spawn((
                TopScoresText,
                Text::new("Top scores\n—"),
                TextFont {
                    font_size: 15.0,
                    ..default()
                },
                TextColor(Color::srgba(0.82, 0.88, 0.96, 0.95)),
            ));
        });
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
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(10.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
            Visibility::Visible,
        ))
        .with_children(|p| {
            p.spawn((
                GetReadyBodyName,
                Text::new(""),
                TextLayout::new_with_justify(Justify::Center),
                TextFont {
                    font_size: 30.0,
                    ..default()
                },
                TextColor(Color::srgba(0.88, 0.93, 1.0, 0.0)),
            ));
            p.spawn((
                GetReadyText,
                Text::new("Get Ready"),
                TextLayout::new_with_justify(Justify::Center),
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
        Query<&mut Visibility, With<TopScoresRoot>>,
    )>,
) {
    let playing = *state.get() == AppState::Playing;
    let get_ready = *state.get() == AppState::GetReady;
    let landing_success = *state.get() == AppState::LandingSuccess;
    let game_over = *state.get() == AppState::GameOver;
    let show_top_scores = get_ready || game_over;

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
    for mut v in q.p3() {
        *v = if show_top_scores {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn update_hud(
    body: Res<CurrentBody>,
    level_timer: Res<LevelFlightTimer>,
    run_score: Res<RunScore>,
    ship: Query<
        &Ship,
        (
            With<ShipRoot>,
            Without<HudPlanet>,
            Without<HudFuel>,
            Without<HudVel>,
            Without<HudTimer>,
            Without<HudRunScore>,
            Without<HudGameOver>,
        ),
    >,
    mut fuel_q: Query<
        (&mut Text, &mut TextColor),
        (
            With<HudFuel>,
            Without<HudVel>,
            Without<HudPlanet>,
            Without<HudTimer>,
            Without<HudRunScore>,
            Without<HudGameOver>,
        ),
    >,
    mut vel_q: Query<
        (&mut Text, &mut TextColor),
        (
            With<HudVel>,
            Without<HudFuel>,
            Without<HudPlanet>,
            Without<HudTimer>,
            Without<HudRunScore>,
            Without<HudGameOver>,
        ),
    >,
    mut planet_q: Query<
        &mut Text,
        (
            With<HudPlanet>,
            Without<HudFuel>,
            Without<HudVel>,
            Without<HudTimer>,
            Without<HudRunScore>,
            Without<HudGameOver>,
        ),
    >,
    mut timer_q: Query<
        &mut Text,
        (
            With<HudTimer>,
            Without<HudPlanet>,
            Without<HudFuel>,
            Without<HudVel>,
            Without<HudRunScore>,
            Without<HudGameOver>,
        ),
    >,
    mut run_score_q: Query<
        &mut Text,
        (
            With<HudRunScore>,
            Without<HudPlanet>,
            Without<HudFuel>,
            Without<HudVel>,
            Without<HudTimer>,
            Without<HudGameOver>,
        ),
    >,
) {
    let Ok(ship) = ship.single() else {
        return;
    };

    let g = body.0.real_gravity_m_s2();
    for mut t in &mut planet_q {
        t.0 = format!("{} ({:.2} m/s^2)", body.0.display_name(), g);
    }

    for mut t in &mut timer_q {
        t.0 = format!("Time: {:.1} s", level_timer.elapsed);
    }
    for mut t in &mut run_score_q {
        t.0 = format!("Score: {}", run_score.0);
    }

    let fuel_color = if body.0 == CelestialBody::Jupiter {
        Color::srgb(0.82, 0.62, 1.0)
    } else {
        Color::srgb(0.7, 1.0, 0.75)
    };
    for (mut t, mut tc) in &mut fuel_q {
        t.0 = format!(
            "{}: {:>3.0}%",
            fuel_display_name(body.0),
            (ship.fuel / INITIAL_FUEL * 100.0).clamp(0.0, 100.0)
        );
        **tc = fuel_color;
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
        t.0 = format!("{:+7.1} m/s", vy_ms);
        **tc = vel_color;
    }
}

fn update_top_scores_text(
    high_scores: Res<HighScores>,
    mut q: Query<&mut Text, With<TopScoresText>>,
) {
    let Ok(mut text) = q.single_mut() else {
        return;
    };
    let entries: Vec<_> = high_scores
        .0
        .iter()
        .filter(|e| e.score > 0)
        .take(5)
        .collect();
    if entries.is_empty() {
        text.0 = "Top scores\n(no runs yet)".to_string();
        return;
    }
    let mut s = String::from("Top scores\n");
    for (i, e) in entries.iter().enumerate() {
        if let Some(ts) = format_high_score_timestamp(e.unix_secs) {
            s.push_str(&format!("{}. {}  {}\n", i + 1, e.score, ts));
        } else {
            s.push_str(&format!("{}. {}\n", i + 1, e.score));
        }
    }
    if s.ends_with('\n') {
        s.pop();
    }
    text.0 = s;
}

fn update_game_over_overlay(
    state: Res<State<AppState>>,
    game_end: Res<GameEnd>,
    mut text_q: Query<&mut Text, With<HudGameOver>>,
    mut root_vis: Query<&mut Visibility, With<HudGameOverRoot>>,
) {
    let Ok(mut text) = text_q.single_mut() else {
        return;
    };
    let Ok(mut vis) = root_vis.single_mut() else {
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
