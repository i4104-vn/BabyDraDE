//! Audio and microphone mute action handlers.

use crate::center::features::recording::service;
use crate::center::features::recording::ui::RecordingButtonWidget;

/// Toggles audio mute and updates the button visual.
pub fn toggle_audio_mute_action(button: &RecordingButtonWidget) {
    let muted = service::toggle_audio_mute();
    button.set_icon_and_title(
        if muted {
            "audio-volume-muted"
        } else {
            "audio-volume-high"
        },
        &services::i18n::trans(if muted {
            "recorder.audio_unmute"
        } else {
            "recorder.audio_mute"
        }),
    );
    button.set_alert(muted);
}

/// Toggles microphone mute and updates the button visual.
pub fn toggle_mic_mute_action(button: &RecordingButtonWidget) {
    let muted = service::toggle_mic_mute();
    button.set_icon_and_title(
        if muted {
            "microphone-disabled"
        } else {
            "audio-input-microphone"
        },
        &services::i18n::trans(if muted {
            "recorder.mic_unmute"
        } else {
            "recorder.mic_mute"
        }),
    );
    button.set_alert(muted);
}
