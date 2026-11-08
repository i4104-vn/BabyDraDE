mod css;

use gtk4::gdk::Display;
use gtk4::{CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION};

pub use css::TOPBAR_CSS;

pub fn load_topbar_style() {
    let provider = CssProvider::new();
    provider.load_from_data(TOPBAR_CSS);

    if let Some(display) = Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

