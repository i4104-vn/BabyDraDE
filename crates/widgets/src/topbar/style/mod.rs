//! TopBar Style Module — delegates to Central UI Kit Theme Engine

/// Initializes the topbar style via the central UI Kit Theme Engine
pub fn load_topbar_style() {
    ui_kit::init_theme();
}
