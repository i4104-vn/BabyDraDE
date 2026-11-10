//! Styling module — delegates to Central Theme Engine

pub mod theme {
    pub use crate::theme::*;
}

pub use crate::theme::{
    build_css, get_active_css, init_theme, is_dark_mode, load_ui_kit_styles, reload_theme,
    set_dark_mode, DARK_CSS, LIGHT_CSS, SHARED_CSS,
};

use gtk4::gdk::Display;
use gtk4::{CssProvider, STYLE_PROVIDER_PRIORITY_USER};

/// Appends custom CSS stylesheet data to the current display
pub fn append_custom_css(custom_css: &str) {
    let provider = CssProvider::new();
    provider.load_from_data(custom_css);

    if let Some(display) = Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            STYLE_PROVIDER_PRIORITY_USER,
        );
    }
}
