//! BabyDraDE Central Theme & Styling Engine
//!
//! Provides a single source of truth for all styling, layout, and dark/light mode
//! switching driven by GTK configuration.

pub mod dark_css;
pub mod engine;
pub mod light_css;
pub mod shared_css;

pub use dark_css::DARK_CSS;
pub use engine::{build_css, get_active_css, init_theme, is_dark_mode, reload_theme, set_dark_mode};
pub use light_css::LIGHT_CSS;
pub use shared_css::SHARED_CSS;

/// Backwards-compatible alias for `init_theme`.
pub fn load_ui_kit_styles() {
    init_theme();
}

