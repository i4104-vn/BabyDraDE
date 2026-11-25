//! Control Center UI module for BabyDraDE TopBar

pub mod modal;
pub mod modules;

// Re-export modules as module for convenience / compatibility
pub use modules as module;

// Re-export all submodules so existing imports remain backward-compatible
pub use modules::*;

pub use modal::{
    close_all_panels, create_cc_window, register_panel, setup_click_outside_dismiss,
    sync_cc_brightness, sync_cc_volume, unregister_panel, WindowSlot,
};
