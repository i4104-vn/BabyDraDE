//! Data source for the clipboard island feature.
//!
//! IPC triggers are handled centrally by `crate::island::ipc`; this module only
//! reads/writes the clipboard cache.

use std::io::Write;
use std::process::{Command, Stdio};

pub use services::system::clipboard::{history, ClipboardEntry};

/// Returns the cached clipboard history.
pub fn get_entries() -> Vec<ClipboardEntry> {
    history()
}

/// Whether clipboard history tracking is enabled.
pub fn is_enabled() -> bool {
    services::config::load_clipboard_config().enabled
}

/// Cheap content revision used to detect history changes between ticks.
pub fn revision() -> u64 {
    let mut hash = 14_695_981_039_346_656_037u64;
    for entry in history() {
        for byte in entry.preview.as_bytes() {
            hash = (hash ^ u64::from(*byte)).wrapping_mul(1_099_511_628_211);
        }
        hash = (hash ^ entry.timestamp).wrapping_mul(1_099_511_628_211);
    }
    hash
}

/// Copies a history entry back to the system clipboard and refreshes the cache.
pub fn copy_to_system_clipboard(entry: &ClipboardEntry) {
    if let Ok(mut child) = Command::new("wl-copy").stdin(Stdio::piped()).spawn() {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(&entry.data);
        }
    }
    services::system::clipboard::store_selection(&entry.mime, &[], &entry.data);
}
