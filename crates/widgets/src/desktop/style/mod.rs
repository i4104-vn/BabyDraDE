//! Desktop Shell Style Module — delegates to Central UI Kit Theme Engine

/// Initializes the desktop shell style via the central UI Kit Theme Engine
pub fn load_desktop_style() {
    ui_kit::init_theme();
}
