//! BabyDraDE Desktop Widgets Library
//!
//! Provides the core implementation for BabyDraDE desktop shell widgets:
//! desktop grid/wallpaper and topbar/panel widgets.

pub mod common;
pub mod desktop;
pub mod topbar;

/// Available built-in widget identifiers
pub const AVAILABLE_WIDGETS: &[&str] = &["desktop", "topbar", "panel"];

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
        other => Err(format!(
            "Unknown widget '{}'. Available widgets: {:?}",
            other, AVAILABLE_WIDGETS
        )
        .into()),
    }
}
