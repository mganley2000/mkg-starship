//! Persist top scores: `localStorage` on WASM, file on native.

#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

use chrono::{Local, TimeZone};

#[cfg(target_arch = "wasm32")]
const LOCAL_STORAGE_KEY: &str = "mkg_starship_top_scores_v2";
#[cfg(target_arch = "wasm32")]
const LOCAL_STORAGE_KEY_LEGACY: &str = "mkg_starship_top_scores_v1";
const MAX_SCORES: usize = 5;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HighScoreEntry {
    pub score: i32,
    /// Unix timestamp (seconds) when the run ended; `0` = unknown (legacy saves).
    pub unix_secs: i64,
}

/// Human-readable local date/time for a run; [`None`] if unknown.
pub fn format_high_score_timestamp(unix_secs: i64) -> Option<String> {
    if unix_secs <= 0 {
        return None;
    }
    Local
        .timestamp_opt(unix_secs, 0)
        .single()
        .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
        .or_else(|| {
            chrono::DateTime::<chrono::Utc>::from_timestamp(unix_secs, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M UTC").to_string())
        })
}

fn normalize(mut v: Vec<HighScoreEntry>) -> Vec<HighScoreEntry> {
    v.retain(|e| e.score > 0);
    v.sort_by(|a, b| b.score.cmp(&a.score));
    v.truncate(MAX_SCORES);
    v
}

fn serialize(entries: &[HighScoreEntry]) -> String {
    let mut s = String::from("v2\n");
    for e in entries {
        if e.score > 0 {
            s.push_str(&format!("{},{}\n", e.score, e.unix_secs));
        }
    }
    s
}

fn deserialize(raw: &str) -> Vec<HighScoreEntry> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return vec![];
    }

    if trimmed.starts_with("v2") {
        let mut out = Vec::new();
        for line in trimmed.lines().skip(1) {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let mut parts = line.split(',');
            if let (Some(s), Some(t)) = (parts.next(), parts.next()) {
                if let (Ok(score), Ok(unix_secs)) =
                    (s.trim().parse::<i32>(), t.trim().parse::<i64>())
                {
                    if score > 0 {
                        out.push(HighScoreEntry { score, unix_secs });
                    }
                }
            }
        }
        return normalize(out);
    }

    // Legacy: comma-separated scores only (v1).
    let legacy: Vec<i32> = trimmed
        .split(',')
        .filter_map(|p| p.trim().parse().ok())
        .filter(|&n| n > 0)
        .collect();
    normalize(
        legacy
            .into_iter()
            .map(|score| HighScoreEntry {
                score,
                unix_secs: 0,
            })
            .collect(),
    )
}

/// Load up to [`MAX_SCORES`] best scores, descending (non-zero only).
pub fn load_high_scores() -> Vec<HighScoreEntry> {
    #[cfg(target_arch = "wasm32")]
    {
        return load_wasm();
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        return load_native();
    }
}

pub fn save_high_scores(entries: &[HighScoreEntry]) {
    let v = normalize(entries.iter().cloned().collect());
    let s = serialize(&v);

    #[cfg(target_arch = "wasm32")]
    save_wasm(&s);

    #[cfg(not(target_arch = "wasm32"))]
    save_native(&s);
}

/// Insert a finished run; ignores non-positive scores. Keeps top [`MAX_SCORES`], persists.
pub fn merge_and_persist(prev: &mut Vec<HighScoreEntry>, new_score: i32) {
    if new_score <= 0 {
        return;
    }
    let ts = chrono::Utc::now().timestamp();
    prev.push(HighScoreEntry {
        score: new_score,
        unix_secs: ts,
    });
    *prev = normalize(std::mem::take(prev));
    save_high_scores(prev);
}

#[cfg(target_arch = "wasm32")]
fn load_wasm() -> Vec<HighScoreEntry> {
    use web_sys::window;
    let Some(w) = window() else {
        return vec![];
    };
    let Ok(Some(storage)) = w.local_storage() else {
        return vec![];
    };
    let raw = storage
        .get_item(LOCAL_STORAGE_KEY)
        .ok()
        .flatten()
        .or_else(|| storage.get_item(LOCAL_STORAGE_KEY_LEGACY).ok().flatten());
    let Some(raw) = raw else {
        return vec![];
    };
    deserialize(&raw)
}

#[cfg(target_arch = "wasm32")]
fn save_wasm(s: &str) {
    use web_sys::window;
    let Some(w) = window() else {
        return;
    };
    let Ok(Some(storage)) = w.local_storage() else {
        return;
    };
    let _ = storage.set_item(LOCAL_STORAGE_KEY, s);
}

#[cfg(not(target_arch = "wasm32"))]
fn data_path() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        std::env::var("APPDATA").ok().map(|p| {
            PathBuf::from(p)
                .join("mkg-starship")
                .join("highscores.txt")
        })
    }
    #[cfg(not(windows))]
    {
        std::env::var("HOME").ok().map(|p| {
            PathBuf::from(p)
                .join(".local/share/mkg-starship")
                .join("highscores.txt")
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn load_native() -> Vec<HighScoreEntry> {
    let Some(path) = data_path() else {
        return vec![];
    };
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return vec![];
    };
    deserialize(raw.trim())
}

#[cfg(not(target_arch = "wasm32"))]
fn save_native(s: &str) {
    let Some(path) = data_path() else {
        return;
    };
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(&path, s);
}
