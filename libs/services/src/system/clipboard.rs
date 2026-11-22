//! Persistent clipboard cache and history.
//!
//! The compositor caches the data of client-owned selections so that copying
//! still works after the source application exits. The same cache backs the
//! clipboard history picker opened with Super+V.

use std::{
    fs,
    path::PathBuf,
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

/// One entry in the clipboard history.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ClipboardEntry {
    /// Primary MIME type of the selection.
    pub mime: String,
    /// Raw selection bytes.
    pub data: Vec<u8>,
    /// Short UTF-8 preview shown in the picker.
    pub preview: String,
    /// Unix timestamp (milliseconds) when the entry was captured.
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
struct CurrentSelection {
    mime: String,
    mime_types: Vec<String>,
}

fn clipboard_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("babydra")
        .join("clipboard")
}

fn current_data_path() -> PathBuf {
    clipboard_dir().join("current.bin")
}

fn current_meta_path() -> PathBuf {
    clipboard_dir().join("current.json")
}

fn history_path() -> PathBuf {
    clipboard_dir().join("history.json")
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Creates the clipboard cache directory if needed.
pub fn ensure_dir() {
    let _ = fs::create_dir_all(clipboard_dir());
}

fn preview_for(mime: &str, data: &[u8]) -> String {
    if mime.starts_with("text/") || mime == "UTF8_STRING" || mime == "STRING" {
        String::from_utf8_lossy(data).chars().take(240).collect()
    } else {
        format!("<{} bytes>", data.len())
    }
}

/// Caches a freshly captured selection and appends it to the history.
pub fn store_selection(mime: &str, mime_types: &[String], data: &[u8]) {
    ensure_dir();

    let _ = fs::write(current_data_path(), data);
    let meta = CurrentSelection {
        mime: mime.to_string(),
        mime_types: mime_types.to_vec(),
    };
    if let Ok(json) = serde_json::to_string(&meta) {
        let _ = fs::write(current_meta_path(), json);
    }

    let config = crate::config::load_clipboard_config();
    if !config.enabled || data.is_empty() {
        return;
    }

    let preview = preview_for(mime, data);
    if preview.is_empty() {
        return;
    }

    let mut entries = history();
    if entries.first().map(|e| e.preview.as_str()) != Some(preview.as_str()) {
        entries.insert(
            0,
            ClipboardEntry {
                mime: mime.to_string(),
                data: data.to_vec(),
                preview,
                timestamp: now_millis(),
            },
        );
        let max_items = config.max_items.max(1);
        entries.truncate(max_items);
        if let Ok(json) = serde_json::to_string(&entries) {
            let _ = fs::write(history_path(), json);
        }
    }
}

/// Reads the cached selection immediately, if available.
pub fn read_current() -> Option<(String, Vec<u8>)> {
    let meta: CurrentSelection =
        serde_json::from_str(&fs::read_to_string(current_meta_path()).ok()?).ok()?;
    let data = fs::read(current_data_path()).ok()?;
    Some((meta.mime, data))
}

/// Waits briefly for the asynchronous selection capture to finish and then
/// reads the cached bytes for `mime_type`.
pub fn wait_and_read_current(mime_type: &str, timeout: Duration) -> Option<Vec<u8>> {
    let deadline = std::time::Instant::now() + timeout;
    loop {
        if let Some((mime, data)) = read_current() {
            let cached_text = mime.starts_with("text/")
                || matches!(mime.as_str(), "UTF8_STRING" | "STRING");
            let request_text = mime_type.starts_with("text/")
                || matches!(mime_type, "UTF8_STRING" | "STRING");
            // Text MIME names differ between GTK/Qt/XWayland. Any text request
            // can be satisfied by our single cached text representation.
            if mime == mime_type || (cached_text && request_text) {
                return Some(data);
            }
        }
        if std::time::Instant::now() >= deadline {
            // Last resort: if we cached something, serve it rather than
            // leaving the pasting app with an empty selection.
            return read_current().map(|(_, data)| data);
        }
        thread::sleep(Duration::from_millis(5));
    }
}

/// MIME types advertised by the cached selection.
pub fn current_mime_types() -> Vec<String> {
    serde_json::from_str::<CurrentSelection>(&fs::read_to_string(current_meta_path()).unwrap_or_default())
        .map(|meta| meta.mime_types)
        .unwrap_or_default()
}

/// Returns the clipboard history, newest first.
pub fn history() -> Vec<ClipboardEntry> {
    serde_json::from_str(&fs::read_to_string(history_path()).unwrap_or_default()).unwrap_or_default()
}

/// Removes all history entries but keeps the current selection.
pub fn clear_history() {
    let _ = fs::write(history_path(), "[]");
}
