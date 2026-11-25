//! Adapter for the recorder backend.
//!
//! `babydra-island` talked to `babydra_core::services::recording`, a daemon
//! that does not exist in the BabyDraDE workspace yet. This module keeps the
//! exact API surface used by the island feature/UI so the full recorder
//! controls are present; until a backend is wired in it reports an idle
//! session and the action calls are safe no-ops.

use std::path::PathBuf;

use crate::center::features::recording::models::RecordingConfig;

/// Whether a recording session is currently active.
pub fn is_recording() -> bool {
    false
}

/// Whether the active session is paused.
pub fn is_paused() -> bool {
    false
}

/// Elapsed seconds of the active session.
pub fn get_elapsed_secs() -> u64 {
    0
}

/// Directory where finished recordings are stored.
pub fn get_recordings_dir() -> PathBuf {
    dirs::video_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_else(std::env::temp_dir)
}

/// Opens an interactive area selector (`slurp`) and returns its geometry.
pub fn select_geometry_str_with_slurp() -> Option<String> {
    let output = std::process::Command::new("slurp").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let geometry = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!geometry.is_empty()).then_some(geometry)
}

/// Starts a recording session with the given configuration.
pub fn start_recording(_config: &RecordingConfig) -> Result<(), ()> {
    // No in-process recorder backend is registered in the topbar build.
    Err(())
}

/// Stops the active recording session.
pub fn stop_recording() -> Result<(), ()> {
    Err(())
}

/// Toggles pause/resume.
pub fn toggle_pause() -> Result<(), ()> {
    Err(())
}

/// Whether the recorded system audio stream is muted.
pub fn is_recording_audio_muted() -> bool {
    false
}

/// Whether the recorded microphone stream is muted.
pub fn is_recording_mic_muted() -> bool {
    false
}

/// Toggles system-audio mute and returns the new state.
pub fn toggle_recording_audio_mute() -> bool {
    false
}

/// Toggles microphone mute and returns the new state.
pub fn toggle_recording_mic_mute() -> bool {
    false
}

