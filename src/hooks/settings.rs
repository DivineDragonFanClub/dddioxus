//! Tiny on-disk settings so the app remembers a few things between runs.
//! Stored as JSON in the OS config dir, best-effort (any IO error is ignored,
//! we just fall back to defaults).

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
struct Settings {
    // last host we connected to, so the connect-by-IP box can prefill it
    #[serde(default)]
    last_host: String,
}

fn settings_path() -> Option<PathBuf> {
    let mut dir = dirs::config_dir()?;
    dir.push("dddioxus");
    Some(dir.join("settings.json"))
}

fn load() -> Settings {
    settings_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn store(settings: &Settings) {
    let Some(path) = settings_path() else { return };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(settings) {
        let _ = std::fs::write(path, json);
    }
}

/// The last host we connected to, if any.
pub fn last_host() -> Option<String> {
    let host = load().last_host;
    (!host.is_empty()).then_some(host)
}

/// Remember the host we just connected to.
pub fn set_last_host(host: &str) {
    let mut settings = load();
    if settings.last_host == host {
        return;
    }
    settings.last_host = host.to_string();
    store(&settings);
}
