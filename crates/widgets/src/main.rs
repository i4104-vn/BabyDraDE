//! BabyDraDE Built-in Desktop Widgets Executable

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();

    let mut args = std::env::args().skip(1);
    let mut widget_name = args.next().unwrap_or_else(|| "topbar".to_string());
    if widget_name == "__widget" {
        widget_name = args.next().unwrap_or_else(|| "topbar".to_string());
    }
    run_widget(&widget_name)
}
