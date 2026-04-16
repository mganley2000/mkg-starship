//! Optional looping background music (native only). You are responsible for having rights to any
//! file you place under `assets/music/` (see `setup_ambient_music`).
#![cfg_attr(target_arch = "wasm32", allow(dead_code))]

use bevy::audio::{
    AudioPlayer, AudioSink, AudioSinkPlayback, AudioSource, GlobalVolume, PlaybackMode,
    PlaybackSettings, Volume,
};
use bevy::log::warn;
use bevy::prelude::*;

/// Marker for the ambient music entity.
#[derive(Component)]
struct AmbientMusicPlayer;

/// Linear gain for background music (multiplied by [`GlobalVolume`] each frame).
const AMBIENT_MUSIC_LINEAR: f32 = 0.14;

/// Prefer OGG (smaller); MP3 if you only have that format. Place **one** of these next to
/// `Cargo.toml`: `assets/music/bgm.ogg` or `assets/music/bgm.mp3`.
fn resolve_bgm_asset_path() -> Option<&'static str> {
    if std::path::Path::new("assets/music/bgm.ogg").is_file() {
        return Some("music/bgm.ogg");
    }
    if std::path::Path::new("assets/music/bgm.mp3").is_file() {
        return Some("music/bgm.mp3");
    }
    None
}

fn setup_ambient_music(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut warned: Local<bool>,
) {
    let Some(path) = resolve_bgm_asset_path() else {
        if !*warned {
            warn!(
                "Ambient music disabled: add assets/music/bgm.ogg or assets/music/bgm.mp3 \
                 (you must have rights to redistribute or use that audio)."
            );
            *warned = true;
        }
        return;
    };

    let handle: Handle<AudioSource> = asset_server.load(path);
    commands.spawn((
        AmbientMusicPlayer,
        AudioPlayer::new(handle),
        PlaybackSettings {
            mode: PlaybackMode::Loop,
            volume: Volume::Linear(0.0),
            paused: false,
            ..default()
        },
    ));
}

fn update_ambient_music_volume(
    global_volume: Res<GlobalVolume>,
    mut sinks: Query<&mut AudioSink, With<AmbientMusicPlayer>>,
) {
    let Ok(mut sink) = sinks.single_mut() else {
        return;
    };
    let v = Volume::Linear(AMBIENT_MUSIC_LINEAR) * global_volume.volume;
    sink.set_volume(v);
}

pub struct AmbientMusicPlugin;

impl Plugin for AmbientMusicPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            app.add_systems(Startup, setup_ambient_music).add_systems(Last, update_ambient_music_volume);
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = app;
        }
    }
}
