//! User action dispatchers for the recording island feature.

use super::recording::{
    is_recording_audio_muted, is_recording_mic_muted, toggle_recording_audio_mute,
    toggle_recording_mic_mute,
};

/// Toggles recording audio mute and returns the new state.
pub fn toggle_audio_mute() -> bool {
    toggle_recording_audio_mute()
}

/// Toggles recording microphone mute and returns the new state.
pub fn toggle_mic_mute() -> bool {
    toggle_recording_mic_mute()
}

/// Returns whether the recorded audio stream is muted.
pub fn is_audio_muted() -> bool {
    is_recording_audio_muted()
}

/// Returns whether the recorded microphone stream is muted.
pub fn is_mic_muted() -> bool {
    is_recording_mic_muted()
}
