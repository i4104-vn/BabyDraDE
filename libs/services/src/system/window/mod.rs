//! Active window tracking and window actions for BabyDraDE features.
//!
//! Focused-window metadata is still published as a JSON file by the compositor.
//! Window *actions* travel through the typed `protocol` bridge — no action file
//! is read or written any more.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

use protocol::{
    client::FeatureBridge,
    models::{WindowAction, WindowTarget},
};
use serde::{Deserialize, Serialize};

/// Metadata for the currently focused window.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ActiveWindowInfo {
    /// Wayland app id of the focused toplevel.
    pub app_id: String,
    /// Human-readable window title.
    pub title: String,
    /// Process id, when the compositor could resolve it.
    pub process_id: Option<u32>,
}

impl ActiveWindowInfo {
    /// Returns `true` when no window is currently focused.
    pub fn is_empty(&self) -> bool {
        self.app_id.trim().is_empty() && self.title.trim().is_empty()
    }
}

fn get_active_window_file_path() -> PathBuf {
    if let Ok(dir) = std::env::var("XDG_RUNTIME_DIR") {
        let path = PathBuf::from(dir);
        if path.is_dir() {
            return path.join("babydra-active-window.json");
        }
    }
    PathBuf::from("/tmp/babydra-active-window.json")
}

/// Reads the focused-window snapshot published by the compositor.
pub fn read_active_window_file() -> Option<ActiveWindowInfo> {
    let path = get_active_window_file_path();
    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(info) = serde_json::from_str::<ActiveWindowInfo>(&content) {
            return Some(info);
        }
    }
    None
}

/// Saves the focused-window snapshot (compositor side).
pub fn save_active_window(info: &ActiveWindowInfo) {
    let path = get_active_window_file_path();
    if let Ok(json) = serde_json::to_string(info) {
        let _ = fs::write(path, json);
    }
}

/// Queries the focused window, falling back to `wlrctl`.
pub fn get_active_window() -> Option<ActiveWindowInfo> {
    read_active_window_file().or_else(query_wlrctl_active_window)
}

fn query_wlrctl_active_window() -> Option<ActiveWindowInfo> {
    let output = Command::new("wlrctl")
        .args(["window", "list", "state:focused"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout.lines().next()?;
    let (app_id, title) = line.split_once(':')?;
    let app_id = app_id.trim().to_string();
    let title = title.trim().to_string();
    if app_id.is_empty() && title.is_empty() {
        return None;
    }
    Some(ActiveWindowInfo {
        app_id,
        title,
        process_id: None,
    })
}

/// Formats a user-friendly application name from app id and title.
pub fn get_app_display_name(app_id: &str, title: &str) -> String {
    let trimmed_id = app_id.trim().strip_suffix(".desktop").unwrap_or(app_id.trim());
    if !trimmed_id.is_empty() {
        let name = trimmed_id.split('.').next_back().unwrap_or(trimmed_id);
        let mut chars = name.chars();
        if let Some(first) = chars.next() {
            return format!("{}{}", first.to_uppercase(), chars.as_str());
        }
    }
    let trimmed_title = title.trim();
    if let Some((_, app_part)) = trimmed_title.rsplit_once(" — ") {
        return app_part.trim().to_string();
    }
    if let Some((_, app_part)) = trimmed_title.rsplit_once(" - ") {
        return app_part.trim().to_string();
    }
    trimmed_title.to_string()
}

/// Feature-side bridge used for all window actions.
fn bridge() -> &'static FeatureBridge {
    static BRIDGE: OnceLock<FeatureBridge> = OnceLock::new();
    BRIDGE.get_or_init(|| FeatureBridge::new("org.babydrade.services"))
}

/// Sends a window action through the protocol bridge.
///
/// # Arguments
/// * `action` - Legacy action name (`close`, `minimize`, `maximize`, `fullscreen`).
pub fn send_window_action(action: &str) {
    let Some(action) = WindowAction::parse(action) else {
        return;
    };
    let _ = bridge().window_action(action);
}

/// Closes the window identified by `app_id` or `title`, or the focused one.
pub fn close_window_by_app(app_id: &str, title: &str) {
    let target = if !app_id.is_empty() {
        WindowTarget::AppId(app_id.to_string())
    } else if !title.is_empty() {
        WindowTarget::Title(title.to_string())
    } else {
        WindowTarget::Focused
    };
    let _ = bridge().close_targeted(target);
}

/// Minimizes the currently focused window.
pub fn minimize_active_window() {
    let _ = bridge().window_action(WindowAction::Minimize);
}

/// Maximizes the currently focused window.
pub fn maximize_active_window() {
    let _ = bridge().window_action(WindowAction::Maximize);
}

/// Toggles fullscreen on the currently focused window.
pub fn toggle_fullscreen_active_window() {
    let _ = bridge().window_action(WindowAction::Fullscreen);
}
