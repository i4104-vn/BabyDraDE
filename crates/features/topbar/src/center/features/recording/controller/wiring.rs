//! UI event binding and signal wiring for recording popover.
use gtk4::prelude::*;

use super::actions::{
    open_recordings_dir_action, select_recording_area_action, start_recording_action,
    stop_recording_action, toggle_audio_mute_action, toggle_mic_mute_action, toggle_pause_action,
};
use crate::center::features::recording::models::{PopoverActionsContext, RecordingMode};

/// Connects all UI events, dropdowns, and buttons to the recording controller.
pub fn connect_popover_actions(context: PopoverActionsContext) {
    {
        let config = context.config.clone();
        let display_output_row = context.display_output_row.clone();
        let area_row = context.area_row.clone();
        let output_dropdown = context.output_dropdown.clone();
        let output_names = context.output_names.clone();
        let popover = context.popover.clone();
        let area_label = context.area_label.clone();
        context.mode_combo.connect_selected_notify(move |dropdown| {
            let mut cfg = config.borrow_mut();
            match dropdown.selected() {
                1 => {
                    let output = output_names
                        .get(output_dropdown.selected() as usize)
                        .cloned()
                        .unwrap_or_default();
                    cfg.mode = RecordingMode::SingleOutput(output);
                    display_output_row.set_visible(true);
                    area_row.set_visible(false);
                }
                2 => {
                    cfg.mode = RecordingMode::Area(Default::default());
                    display_output_row.set_visible(false);
                    area_row.set_visible(true);
                    drop(cfg);
                    select_recording_area_action(&popover, &area_label, &config);
                }
                _ => {
                    cfg.mode = RecordingMode::Fullscreen;
                    display_output_row.set_visible(false);
                    area_row.set_visible(false);
                }
            }
        });
    }

    {
        let config = context.config.clone();
        let output_names = context.output_names.clone();
        context
            .output_dropdown
            .connect_selected_notify(move |dropdown| {
                if let Some(name) = output_names.get(dropdown.selected() as usize) {
                    config.borrow_mut().mode = RecordingMode::SingleOutput(name.clone());
                }
            });
    }

    {
        let config = context.config.clone();
        let area_label = context.area_label.clone();
        let popover = context.popover.clone();
        context.area_button.connect_clicked(move |_| {
            select_recording_area_action(&popover, &area_label, &config);
        });
    }

    {
        let config = context.config.clone();
        context.resolution.connect_selected_notify(move |dropdown| {
            config.borrow_mut().resolution = match dropdown.selected() {
                1 => Some((1920, 1080)),
                2 => Some((1280, 720)),
                3 => Some((854, 480)),
                _ => None,
            };
        });
    }

    {
        let config = context.config.clone();
        context.framerate.connect_selected_notify(move |dropdown| {
            config.borrow_mut().framerate = match dropdown.selected() {
                0 => 90,
                2 => 30,
                3 => 24,
                _ => 60,
            };
        });
    }

    {
        let config = context.config.clone();
        context.format.connect_selected_notify(move |dropdown| {
            config.borrow_mut().format = match dropdown.selected() {
                1 => "mkv",
                2 => "webm",
                _ => "mp4",
            }
            .to_string();
        });
    }

    {
        let config = context.config.clone();
        context.hdr.connect_active_notify(move |switch| {
            config.borrow_mut().hdr = switch.is_active();
        });
    }

    {
        let config = context.config.clone();
        let audio_device_names = context.audio_device_names.clone();
        let audio_device = context.audio_device.clone();
        let audio_device_row = context.audio_device_row.clone();
        context.audio.connect_active_notify(move |switch| {
            let active = switch.is_active();
            audio_device_row.set_visible(active);
            audio_device.set_sensitive(active);
            let mut config = config.borrow_mut();
            config.audio = active;
            config.audio_device = if active {
                audio_device_names.first().cloned()
            } else {
                None
            };
        });
    }

    {
        let config = context.config.clone();
        let audio_device_names = context.audio_device_names.clone();
        context
            .audio_device
            .connect_selected_notify(move |dropdown| {
                config.borrow_mut().audio_device = audio_device_names
                    .get(dropdown.selected() as usize)
                    .cloned();
            });
    }

    {
        let config = context.config.clone();
        context.codec.connect_selected_notify(move |dropdown| {
            config.borrow_mut().codec = match dropdown.selected() {
                1 => Some("libx264".to_string()),
                2 => Some("h264_vaapi".to_string()),
                _ => None,
            };
        });
    }

    {
        let config = context.config.clone();
        context.start_button.connect_clicked(move |_| {
            start_recording_action(&config.borrow());
        });
    }

    context
        .open_folder
        .connect_clicked(|_| open_recordings_dir_action());

    context.btn_pause.connect_clicked(|_| toggle_pause_action());

    {
        let timer_label = context.timer_label.clone();
        let settings = context.settings.clone();
        let action_row = context.action_row.clone();
        let live_row = context.live_row.clone();
        let status_badge = context.status_badge.clone();
        context.btn_stop.connect_clicked(move |_| {
            stop_recording_action();
            timer_label.set_text("00:00:00");
            settings.set_visible(true);
            action_row.set_visible(true);
            live_row.set_visible(false);
            status_badge.remove_css_class("badge-recording");
            status_badge.remove_css_class("badge-paused");
            status_badge.add_css_class("badge-ready");
            status_badge.set_text(&services::i18n::trans("recorder.status_idle"));
        });
    }

    {
        let button = context.btn_mute_audio.clone();
        context
            .btn_mute_audio
            .connect_clicked(move |_| toggle_audio_mute_action(&button));
    }

    {
        let button = context.btn_mute_mic.clone();
        context
            .btn_mute_mic
            .connect_clicked(move |_| toggle_mic_mute_action(&button));
    }
}
