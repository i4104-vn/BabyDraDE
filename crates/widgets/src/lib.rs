//! BabyDraDE Desktop Widgets Library
//!
//! Provides the core implementation for BabyDraDE desktop shell widgets:
//! desktop grid/wallpaper, topbar/panel, and screenshot utility.

pub mod common;
pub mod desktop;
pub mod screenshot;
pub mod topbar;

/// Available built-in widget identifiers
pub const AVAILABLE_WIDGETS: &[&str] = &["desktop", "topbar", "panel", "screenshot"];

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
        other => Err(format!(
            "Unknown widget '{}'. Available widgets: {:?}",
            other, AVAILABLE_WIDGETS
        )
        .into()),
    }
}
