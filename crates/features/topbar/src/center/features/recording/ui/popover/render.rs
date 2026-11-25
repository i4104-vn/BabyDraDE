//! Renders recording state into the popover widgets.

use gtk4::prelude::*;

use super::RecordingPopover;
use crate::center::features::recording::service::IslandRecordingState;

impl RecordingPopover {
    /// Restores the settings view shown when no session is active.
    pub(crate) fn reset_to_config(&self) {
        self.timer_label.set_text("00:00:00");
        self.settings.set_visible(true);
        self.action_row.set_visible(true);
        self.live_row.set_visible(false);
        self.status_badge.remove_css_class("badge-recording");
        self.status_badge.remove_css_class("badge-paused");
        self.status_badge.add_css_class("badge-ready");
        self.status_badge
            .set_text(&services::i18n::trans("recorder.status_idle"));
    }

    /// Switches to the live view and syncs timer, badges and controls.
    pub(crate) fn update_data(&self, state: &IslandRecordingState) {
        if !state.is_recording {
            self.reset_to_config();
            return;
        }

        let elapsed = state.elapsed_secs;
        self.timer_label.set_text(&format!(
            "{:02}:{:02}:{:02}",
            elapsed / 3600,
            (elapsed % 3600) / 60,
            elapsed % 60
        ));
        self.settings.set_visible(false);
        self.action_row.set_visible(false);
        self.live_row.set_visible(true);

        self.status_badge.remove_css_class("badge-ready");
        self.status_badge.remove_css_class("badge-recording");
        self.status_badge.remove_css_class("badge-paused");
        if state.is_paused {
            self.status_badge.add_css_class("badge-paused");
            self.status_badge
                .set_text(&services::i18n::trans("recorder.status_paused"));
        } else {
            self.status_badge.add_css_class("badge-recording");
            self.status_badge
                .set_text(&services::i18n::trans("recorder.status_recording"));
        }

        let (pause_icon, pause_hint) = if state.is_paused {
            ("media-playback-start", "recorder.resume")
        } else {
            ("media-playback-pause", "recorder.pause")
        };
        self.btn_pause
            .set_icon_and_title(pause_icon, &services::i18n::trans(pause_hint));

        if !state.mode_name.is_empty() {
            self.mode_label.set_text(&state.mode_name);
        }
        let hdr = if state.hdr { " • HDR10" } else { "" };
        self.quality_label.set_text(&format!(
            "{} FPS • {}{}",
            state.framerate, state.format, hdr
        ));

        self.btn_mute_audio
            .button
            .set_sensitive(state.audio_enabled);
        let audio_icon = if state.is_audio_muted {
            "audio-volume-muted"
        } else {
            "audio-volume-high"
        };
        let audio_hint = if state.is_audio_muted {
            "recorder.audio_unmute"
        } else {
            "recorder.audio_mute"
        };
        self.btn_mute_audio
            .set_icon_and_title(audio_icon, &services::i18n::trans(audio_hint));
        self.btn_mute_audio
            .set_alert(state.is_audio_muted && state.audio_enabled);

        self.btn_mute_mic.button.set_sensitive(state.audio_enabled);
        let mic_icon = if state.is_mic_muted {
            "microphone-disabled"
        } else {
            "audio-input-microphone"
        };
        let mic_hint = if state.is_mic_muted {
            "recorder.mic_unmute"
        } else {
            "recorder.mic_mute"
        };
        self.btn_mute_mic
            .set_icon_and_title(mic_icon, &services::i18n::trans(mic_hint));
        self.btn_mute_mic
            .set_alert(state.is_mic_muted && state.audio_enabled);
    }
}
