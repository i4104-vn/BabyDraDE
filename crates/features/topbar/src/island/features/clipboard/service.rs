//! Data source for the clipboard island feature.
//!
//! IPC triggers are handled centrally by `crate::island::ipc`; this module only
//! reads/writes the clipboard cache.

use std::cell::RefCell;
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub use services::system::clipboard::{history, ClipboardEntry};

/// Hashing every entry on each 250 ms island tick is wasted work; cache the
/// revision briefly and invalidate it immediately after a local mutation.
const REVISION_TTL: Duration = Duration::from_millis(500);
thread_local! {
    static REVISION_CACHE: RefCell<Option<(Instant, u64)>> = const { RefCell::new(None) };
}

/// Returns the cached clipboard history.
pub fn get_entries() -> Vec<ClipboardEntry> {
    history()
}

/// Whether clipboard history tracking is enabled.
pub fn is_enabled() -> bool {
    services::config::load_clipboard_config().enabled
}

/// Content revision used to detect history changes between ticks.
///
/// The underlying hash is cached for [`REVISION_TTL`] so the island's periodic
/// tick does not read and hash the whole history every 250 ms.
pub fn revision() -> u64 {
    REVISION_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if let Some((checked_at, value)) = *cache {
            if checked_at.elapsed() < REVISION_TTL {
                return value;
            }
        }
        let value = compute_revision();
        *cache = Some((Instant::now(), value));
        value
    })
}

fn compute_revision() -> u64 {
    let mut hash = 14_695_981_039_346_656_037u64;
    for entry in history() {
        for byte in entry.preview.as_bytes() {
            hash = (hash ^ u64::from(*byte)).wrapping_mul(1_099_511_628_211);
        }
        hash = (hash ^ entry.timestamp).wrapping_mul(1_099_511_628_211);
    }
    hash
}

/// Forces the next [`revision`] call to recompute from the current history.
pub fn invalidate_revision() {
    REVISION_CACHE.with(|cache| {
        *cache.borrow_mut() = None;
    });
}

/// Copies a history entry back to the system clipboard and refreshes the cache.
pub fn copy_to_system_clipboard(entry: &ClipboardEntry) {
    if let Ok(mut child) = Command::new("wl-copy").stdin(Stdio::piped()).spawn() {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(&entry.data);
        }
    }
    services::system::clipboard::store_selection(&entry.mime, &[], &entry.data);
    invalidate_revision();
}
