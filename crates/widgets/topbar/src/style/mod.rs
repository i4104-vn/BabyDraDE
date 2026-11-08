mod css;

pub use css::TOPBAR_CSS;

pub fn load_topbar_style() {
    // 1. Load base reusable UI Kit styles
    ui_kit::load_ui_kit_styles();

    // 2. Load topbar-specific styles
    ui_kit::style::append_custom_css(TOPBAR_CSS);
}
