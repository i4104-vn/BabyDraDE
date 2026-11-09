//! BabyDraDE Built-in Desktop Widgets

pub mod common;
pub mod desktop;
pub mod keymap;
pub mod lock;
pub mod screenshot;
pub mod topbar;

/// Available built-in widget identifiers
pub const AVAILABLE_WIDGETS: &[&str] = &["desktop", "topbar", "screenshot", "keymap", "lock"];

/// Runs a specific built-in widget by name
pub fn run_widget(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    match name {
        "desktop" => {
            desktop::run();
            Ok(())
        }
        "topbar" => {
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
