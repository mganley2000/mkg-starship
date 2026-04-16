//! Looping engine rumble while any thruster is active (native audio only).
#![cfg_attr(target_arch = "wasm32", allow(dead_code))]

use std::f32::consts::PI;

// Single explicit import list — avoids rust-analyzer missing `PlaybackMode` / `Volume` when
// `prelude::*` is combined with separate `use bevy::audio::{...}` lines.
use bevy::audio::{
    AudioPlayer, AudioSink, AudioSinkPlayback, AudioSource, GlobalVolume, PlaybackMode,
    PlaybackSettings, Volume,
};
use bevy::log::warn;
use bevy::prelude::*;

use crate::game_flow::AppState;
use crate::ship::{Ship, ShipRoot};

/// Marker for the global thrust loop entity (not parented to the ship).
#[derive(Component)]
struct ThrustAudioPlayer;

/// PCM WAV bytes: short loop of mixed low/mid sines (~22050 Hz mono); level is set in [`PlaybackSettings`].
fn thrust_loop_wav_bytes() -> Vec<u8> {
    const SAMPLE_RATE: u32 = 22050;
    /// ~0.35 s — seamless enough for a rumble loop.
    const NUM_SAMPLES: usize = 7710;

    let mut pcm = Vec::with_capacity(NUM_SAMPLES * 2);
    for i in 0..NUM_SAMPLES {
        let t = i as f32 / SAMPLE_RATE as f32;
        // Slightly stronger fundamentals + a bit of ~200 Hz body so the hum reads clearly on small speakers.
        let s = 0.09 * (2.0 * PI * 48.0 * t).sin()
            + 0.065 * (2.0 * PI * 92.0 * t).sin()
            + 0.05 * (2.0 * PI * 155.0 * t + 0.8 * t).sin()
            + 0.04 * (2.0 * PI * 210.0 * t).sin();
        let v = (s * 0.58 * 32767.0).clamp(-32768.0, 32767.0) as i16;
        pcm.extend_from_slice(&v.to_le_bytes());
    }

    let data_len = pcm.len();
    let mut out = Vec::with_capacity(44 + data_len);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36u32 + data_len as u32).to_le_bytes());
    out.extend_from_slice(b"WAVE");
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes()); // PCM
    out.extend_from_slice(&1u16.to_le_bytes()); // mono
    out.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    let byte_rate = SAMPLE_RATE * 2;
    out.extend_from_slice(&byte_rate.to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&(data_len as u32).to_le_bytes());
    out.extend_from_slice(&pcm);
    out
}

fn setup_thrust_audio(mut commands: Commands, mut assets: ResMut<Assets<AudioSource>>) {
    let handle = assets.add(AudioSource {
        bytes: thrust_loop_wav_bytes().into(),
    });
    commands.spawn((
        ThrustAudioPlayer,
        AudioPlayer::new(handle),
        PlaybackSettings {
            mode: PlaybackMode::Loop,
            // Start silent; update_thrust_audio sets THRUST_VOLUME × GlobalVolume while thrusting.
            volume: Volume::Linear(0.0),
            paused: false,
            ..default()
        },
    ));
}

/// Perceived level while thrusting (linear × [`GlobalVolume`]).
const THRUST_VOLUME: Volume = Volume::Linear(0.72);

fn update_thrust_audio(
    keyboard: Res<ButtonInput<KeyCode>>,
    ship: Query<&Ship, With<ShipRoot>>,
    global_volume: Res<GlobalVolume>,
    mut sinks: Query<&mut AudioSink, With<ThrustAudioPlayer>>,
    state: Res<State<AppState>>,
    mut thrust_sink_missing_warned: Local<bool>,
) {
    let Ok(mut sink) = sinks.single_mut() else {
        if !*thrust_sink_missing_warned {
            warn!(
                "Thrust audio: no AudioSink (audio device unavailable or sink not created). \
                 Check Windows sound output and that the game is not muted in Volume Mixer."
            );
            *thrust_sink_missing_warned = true;
        }
        return;
    };

    // Must run while not in `Playing` too: the sink kept its last volume after landing because
    // `run_if(in_state(Playing))` skipped this system entirely once we entered `LandingSuccess`.
    if *state.get() != AppState::Playing {
        sink.set_volume(Volume::Linear(0.0));
        sink.pause();
        return;
    }

    let Ok(ship) = ship.single() else {
        sink.set_volume(Volume::Linear(0.0));
        sink.pause();
        return;
    };

    let mut thrust = false;
    if ship.fuel > 0.0 {
        thrust |= keyboard.pressed(KeyCode::ArrowDown);
        thrust |= keyboard.pressed(KeyCode::ArrowLeft);
        thrust |= keyboard.pressed(KeyCode::ArrowRight);
    }

    let v = if thrust {
        THRUST_VOLUME * global_volume.volume
    } else {
        Volume::Linear(0.0)
    };
    sink.set_volume(v);
    if thrust && sink.is_paused() {
        sink.play();
    }
}

pub struct ThrustAudioPlugin;

impl Plugin for ThrustAudioPlugin {
    fn build(&self, app: &mut App) {
        // `bevy_audio` / rodio output is not wired the same on wasm32; skip to keep builds simple.
        #[cfg(not(target_arch = "wasm32"))]
        {
            // `play_queued_audio_system` runs in PostUpdate; we must run after it so `AudioSink` exists.
            // `Last` runs after `PostUpdate` in the main schedule (see Bevy `Main` docs).
            app.add_systems(Startup, setup_thrust_audio).add_systems(Last, update_thrust_audio);
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = app;
        }
    }
}
