//! BabyDraDE Built-in Desktop Widgets

pub mod desktop;
pub mod topbar;

/// Available built-in widget identifiers
pub const AVAILABLE_WIDGETS: &[&str] = &["desktop", "topbar"];

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
        other => Err(format!(
            "Unknown widget '{}'. Available widgets: {:?}",
            other, AVAILABLE_WIDGETS
        )
        .into()),
    }
}
