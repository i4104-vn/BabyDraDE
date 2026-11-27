//! Session lifecycle action handlers (start, stop, pause).

use crate::island::features::recording::models::RecordingConfig;
use crate::island::features::recording::service;

/// Dispatches starting screen recording.
pub fn start_recording_action(config: &RecordingConfig) {
    let _ = service::start_recording(config);
}

/// Dispatches stopping screen recording cleanly on a background worker.
pub fn stop_recording_action() {
    std::thread::spawn(|| {
        let _ = service::stop_recording();
    });
}

/// Dispatches pause / resume toggling.
pub fn toggle_pause_action() {
    let _ = service::toggle_pause();
}
