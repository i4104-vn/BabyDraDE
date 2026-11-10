//! BabyDraDE Desktop Widgets Library
//!
//! Provides the core implementation for BabyDraDE desktop shell widgets:
//! desktop grid/wallpaper, topbar/panel, screenshot utility, keymap daemon, and lock screen.

pub mod common;
pub mod desktop;
pub mod keymap;
pub mod lock;
pub mod screenshot;
pub mod topbar;

/// Available built-in widget identifiers
pub const AVAILABLE_WIDGETS: &[&str] = &[
    "desktop",
    "topbar",
    "panel",
    "screenshot",
    "keymap",
    "lock",
];

/// Runs a specific built-in widget by name
pub fn run_widget(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    match name {
        "desktop" => {
            desktop::run();
            Ok(())
        }
        "topbar" | "panel" => {
            topbar::run();
            Ok(())
        }
        "screenshot" => {
            screenshot::run();
            Ok(())
        }
        "keymap" => {
            keymap::run();
            Ok(())
        }
        "lock" => {
            lock::run();
            Ok(())
        }
        other => Err(format!(
            "Unknown widget '{}'. Available widgets: {:?}",
            other, AVAILABLE_WIDGETS
        )
        .into()),
    }
}

