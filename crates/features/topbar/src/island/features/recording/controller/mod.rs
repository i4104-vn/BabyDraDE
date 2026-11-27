//! Controllers for the recording dynamic island feature.

pub mod actions;
mod input;

pub use crate::island::features::recording::models::PopoverActionsContext;
pub use actions::{
    open_recordings_dir_action, select_recording_area_action, start_recording_action,
    stop_recording_action, toggle_audio_mute_action, toggle_mic_mute_action, toggle_pause_action,
};
pub use input::{keyboard, wiring};
pub use keyboard::create_keyboard_controller;
pub use wiring::connect_popover_actions;
