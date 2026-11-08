pub mod css;
pub mod theme;

use gtk4::gdk::Display;
use gtk4::{CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION};

pub use css::UI_KIT_BASE_CSS;
pub use theme::{ThemePalette, FLUENT_DARK, FLUENT_LIGHT};

/// Loads the UI Kit base styles into GTK4 display
pub fn load_ui_kit_styles() {
    let provider = CssProvider::new();
    provider.load_from_data(UI_KIT_BASE_CSS);

    if let Some(display) = Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

/// Appends custom CSS stylesheet data to the current display
pub fn append_custom_css(custom_css: &str) {
    let provider = CssProvider::new();
    provider.load_from_data(custom_css);

    if let Some(display) = Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            STYLE_PROVIDER_PRIORITY_APPLICATION + 1,
        );
    }
}

