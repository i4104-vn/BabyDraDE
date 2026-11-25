//! Runtime directory layout used by the file transport.

use std::path::PathBuf;

/// Directory name reserved for protocol queues.
pub const PROTOCOL_DIR: &str = "babydra-protocol";
/// Queue carrying feature-to-compositor requests.
pub const REQUESTS_DIR: &str = "requests";
/// Queue carrying compositor-to-feature events.
pub const EVENTS_DIR: &str = "events";
/// Queue carrying responses to features.
pub const RESPONSES_DIR: &str = "responses";

/// Returns `$XDG_RUNTIME_DIR`, falling back to `/tmp` when unset or unusable.
pub fn runtime_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("XDG_RUNTIME_DIR") {
        let path = PathBuf::from(dir);
        if path.is_dir() {
            return path;
        }
    }
    PathBuf::from("/tmp")
}

/// Returns the root directory for all protocol queues.
pub fn protocol_dir() -> PathBuf {
    runtime_dir().join(PROTOCOL_DIR)
}

/// Returns the directory holding one queue.
///
/// # Arguments
/// * `name` - One of `REQUESTS_DIR`, `EVENTS_DIR` or `RESPONSES_DIR`.
pub fn queue_dir(name: &str) -> PathBuf {
    protocol_dir().join(name)
}
