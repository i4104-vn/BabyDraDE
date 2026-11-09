pub mod css;

pub fn load_desktop_style() {
    let provider = gtk4::CssProvider::new();
    provider.load_from_data(css::DESKTOP_CSS);
    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
