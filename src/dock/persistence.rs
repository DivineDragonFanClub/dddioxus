use std::path::PathBuf;

use super::model::DockState;

/// Resolve `~/.dddioxus_layout.json`. Same pattern as `~/.dddioxus_last_host`.
fn layout_path() -> Option<PathBuf> {
    std::env::var("HOME")
        .ok()
        .map(|h| PathBuf::from(h).join(".dddioxus_layout.json"))
}

pub fn load() -> Option<DockState> {
    let path = layout_path()?;
    let s = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&s).ok()
}

pub fn save(state: &DockState) {
    let Some(path) = layout_path() else { return };
    let Ok(s) = serde_json::to_string_pretty(state) else { return };
    let _ = std::fs::write(&path, s);
}
