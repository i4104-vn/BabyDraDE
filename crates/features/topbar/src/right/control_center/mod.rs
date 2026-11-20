//! Control Center UI module for BabyDraDE TopBar

pub mod backlight;
pub mod bluetooth;
pub mod clean;
pub mod header;
pub mod modal;
pub mod storage;
pub mod toggle_grid;
pub mod volume;
pub mod vpn;
pub mod wifi;

pub use modal::{
    close_all_panels, create_cc_window, register_panel, setup_click_outside_dismiss,
    sync_cc_brightness, sync_cc_volume, unregister_panel, WindowSlot,
};
