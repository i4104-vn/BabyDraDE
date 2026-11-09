pub mod css;

pub fn load_desktop_style() {
    // 1. Load base reusable UI Kit styles
    ui_kit::load_ui_kit_styles();

    // 2. Load desktop-specific styles
    ui_kit::style::append_custom_css(css::DESKTOP_CSS);
}
