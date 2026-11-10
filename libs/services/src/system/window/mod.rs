//! Active Window Tracking & Window Management for BabyDraDE
//!
//! Provides real-time detection of the currently focused application and window lifecycle actions
//! (close, minimize, fullscreen).

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ActiveWindowInfo {
    pub app_id: String,
    pub title: String,
    pub process_id: Option<u32>,
}

impl ActiveWindowInfo {
    /// Returns true if no window is currently focused
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

/// Reads the currently active window state written by the compositor
pub fn read_active_window_file() -> Option<ActiveWindowInfo> {
    let path = get_active_window_file_path();
    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(info) = serde_json::from_str::<ActiveWindowInfo>(&content) {
            return Some(info);
        }
    }
    None
}

/// Saves the currently active window state to the runtime directory
pub fn save_active_window(info: &ActiveWindowInfo) {
    let path = get_active_window_file_path();
    if let Ok(json) = serde_json::to_string(info) {
        let _ = fs::write(path, json);
    }
}

/// Queries the focused window with fallback to `wlrctl`
pub fn get_active_window() -> Option<ActiveWindowInfo> {
    // 1. Primary: check compositor active window state file
    if let Some(info) = read_active_window_file() {
        return Some(info);
    }

    // 2. Secondary fallback: query wlrctl window list state:focused
    query_wlrctl_active_window()
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
    if let Some(line) = stdout.lines().next() {
        if let Some(pos) = line.find(':') {
            let app_id = line[..pos].trim().to_string();
            let title = line[pos + 1..].trim().to_string();
            if !app_id.is_empty() || !title.is_empty() {
                return Some(ActiveWindowInfo {
                    app_id,
                    title,
                    process_id: None,
                });
            }
        }
    }
    None
}

/// Formats a user-friendly application display name from app_id and title
pub fn get_app_display_name(app_id: &str, title: &str) -> String {
    let trimmed_id = app_id.trim().strip_suffix(".desktop").unwrap_or(app_id.trim());
    if !trimmed_id.is_empty() {
        // Reverse domain name handling: "org.gnome.TextEditor" -> "TextEditor"
        let name = if let Some(last) = trimmed_id.split('.').next_back() {
            last
        } else {
            trimmed_id
        };
        let mut chars = name.chars();
        if let Some(first) = chars.next() {
            return format!("{}{}", first.to_uppercase(), chars.as_str());
        }
    }

    let trimmed_title = title.trim();
    if !trimmed_title.is_empty() {
        if let Some((_, app_part)) = trimmed_title.rsplit_once(" — ") {
            return app_part.trim().to_string();
        }
        if let Some((_, app_part)) = trimmed_title.rsplit_once(" - ") {
            return app_part.trim().to_string();
        }
        return trimmed_title.to_string();
    }

    String::new()
}

/// Closes the specified window via wlrctl or target specifier
pub fn close_window_by_app(app_id: &str, title: &str) {
    let target = if !app_id.is_empty() {
        format!("app_id:{}", app_id)
    } else if !title.is_empty() {
        format!("title:{}", title)
    } else {
        "state:focused".to_string()
    };
    let _ = Command::new("wlrctl")
        .args(["window", "close", &target])
        .spawn();
}

/// Minimizes the currently focused window
pub fn minimize_active_window() {
    let _ = Command::new("wlrctl")
        .args(["window", "minimize", "state:focused"])
        .spawn();
}

/// Toggles fullscreen for the currently focused window
pub fn toggle_fullscreen_active_window() {
    let _ = Command::new("wlrctl")
        .args(["window", "fullscreen", "state:focused"])
        .spawn();
}
