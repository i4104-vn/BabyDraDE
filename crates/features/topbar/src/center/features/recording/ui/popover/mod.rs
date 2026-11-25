//! Recording configuration and session control popover.

use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label};

use crate::center::features::recording::controller::{
    connect_popover_actions, PopoverActionsContext,
};
use crate::center::features::recording::models::RecordingConfig;
use crate::center::ui::IslandPopover;

pub mod header;
pub mod live;
pub mod render;
pub mod row;
pub mod settings;

pub use header::create_popover_header;
pub use live::create_live_card;
pub use row::setting_row;
pub use settings::create_settings_card;

#[derive(Clone)]
pub struct RecordingPopover {
    pub base: IslandPopover,
    pub timer_label: Label,
    pub status_badge: Label,
    pub mode_label: Label,
    pub quality_label: Label,
    pub btn_pause: crate::center::features::recording::ui::RecordingButtonWidget,
    pub btn_stop: crate::center::features::recording::ui::RecordingButtonWidget,
    pub btn_mute_audio: crate::center::features::recording::ui::RecordingButtonWidget,
    pub btn_mute_mic: crate::center::features::recording::ui::RecordingButtonWidget,
    settings: GtkBox,
    action_row: GtkBox,
    live_row: GtkBox,
}

impl Deref for RecordingPopover {
    type Target = IslandPopover;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl RecordingPopover {
    pub fn new(capsule: &GtkBox) -> Self {
        let base = IslandPopover::new_modal(
            capsule,
            "recording-popover control-popover",
            "recording-popover-box",
            360,
        );
        let config = Rc::new(RefCell::new(RecordingConfig {
            framerate: 90,
            ..Default::default()
        }));

        let (header, status_badge) = create_popover_header();
        base.popover_box.append(&header);

        let settings = create_settings_card();
        base.popover_box.append(&settings.card);
        base.popover_box.append(&settings.action_row);

        let live = create_live_card();
        base.popover_box.append(&live.card);

        connect_popover_actions(PopoverActionsContext {
            popover: base.clone(),
            config,
            mode_combo: settings.mode_dropdown,
            output_dropdown: settings.output_dropdown,
            display_output_row: settings.display_output_row,
            output_names: settings.output_names,
            area_row: settings.area_row,
            area_label: settings.area_label,
            area_button: settings.area_button,
            resolution: settings.resolution,
            framerate: settings.framerate,
            format: settings.format,
            hdr: settings.hdr,
            audio: settings.audio,
            audio_device_row: settings.audio_device_row,
            audio_device: settings.audio_device,
            audio_device_names: settings.audio_device_names,
            codec: settings.codec,
            start_button: settings.start_button,
            open_folder: settings.open_folder,
            btn_pause: live.btn_pause.clone(),
            btn_stop: live.btn_stop.clone(),
            btn_mute_audio: live.btn_mute_audio.clone(),
            btn_mute_mic: live.btn_mute_mic.clone(),
            timer_label: live.timer_label.clone(),
            settings: settings.card.clone(),
            action_row: settings.action_row.clone(),
            live_row: live.card.clone(),
            status_badge: status_badge.clone(),
        });

        Self {
            base,
            timer_label: live.timer_label,
            status_badge,
            mode_label: live.mode_label,
            quality_label: live.quality_label,
            btn_pause: live.btn_pause,
            btn_stop: live.btn_stop,
            btn_mute_audio: live.btn_mute_audio,
            btn_mute_mic: live.btn_mute_mic,
            settings: settings.card,
            action_row: settings.action_row,
            live_row: live.card,
        }
    }
}
