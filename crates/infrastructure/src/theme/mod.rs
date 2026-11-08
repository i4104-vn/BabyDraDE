pub mod cursor;
pub mod gtk;
pub mod icons;
pub mod model;

pub use cursor::{
    find_cursor_theme_path, list_available_cursor_themes, read_active_cursor_theme,
    read_default_index_theme, CursorThemeInfo,
};
pub use gtk::{
    find_gtk4_css_path, find_settings_ini_path, parse_settings_ini, read_gtk4_css,
    read_gtk_settings,
};
pub use icons::{find_icon_theme_path, list_available_icon_themes};
pub use model::GtkThemeConfig;

#[derive(Debug, Clone, Default)]
pub struct GtkThemeReader;

impl GtkThemeReader {
    pub fn new() -> Self {
        Self
    }

    /// Reads and parses the user's current GTK4 / GTK3 configuration (~/.config/gtk-4.0/settings.ini)
    pub fn read(&self) -> GtkThemeConfig {
        read_gtk_settings()
    }

    /// Reads the currently configured cursor theme and size
    pub fn read_cursor(&self) -> CursorThemeInfo {
        read_active_cursor_theme()
    }

    /// Reads custom GTK4 CSS (~/.config/gtk-4.0/gtk.css) if present
    pub fn read_css(&self) -> Option<String> {
        read_gtk4_css()
    }
}
