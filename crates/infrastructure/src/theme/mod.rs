pub mod cursor;
pub mod gtk;
pub mod icons;
pub mod model;

pub use cursor::{apply_cursor_theme, list_available_cursor_themes};
pub use gtk::{apply_gtk4_css, apply_gtk_settings, read_current_gtk4_css, read_current_gtk4_settings, remove_gtk4_css, sync_gsettings};
pub use icons::list_available_icon_themes;
pub use model::{ColorScheme, ThemeConfig};

use std::io;
use tracing::info;

pub struct ThemeService {
    config: ThemeConfig,
}

impl Default for ThemeService {
    fn default() -> Self {
        Self::new(ThemeConfig::default())
    }
}

impl ThemeService {
    pub fn new(config: ThemeConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &ThemeConfig {
        &self.config
    }

    pub fn set_config(&mut self, config: ThemeConfig) {
        self.config = config;
    }

    /// Synchronizes GTK4, GTK3, Cursor, Icons, and Environment across the desktop session
    pub fn apply(&self) -> io::Result<()> {
        info!(
            "Applying GTK4 Theme: '{}', Icon: '{}', Cursor: '{}' ({})",
            self.config.gtk_theme,
            self.config.icon_theme,
            self.config.cursor_theme,
            self.config.cursor_size
        );

        // 1. Export session environment variables so child processes inherit them
        std::env::set_var("GTK_THEME", &self.config.gtk_theme);

        // 2. Apply GTK4 and GTK3 settings.ini files
        apply_gtk_settings(&self.config)?;

        // 3. Apply custom GTK4 CSS if configured
        if let Some(ref css) = self.config.custom_css {
            let _ = apply_gtk4_css(css);
        }

        // 4. Apply Cursor theme (~/.icons/default/index.theme + XCURSOR env vars)
        let _ = apply_cursor_theme(&self.config.cursor_theme, self.config.cursor_size);

        // 5. Trigger non-blocking gsettings sync for GNOME/GTK desktop interface
        sync_gsettings(&self.config);

        info!("GTK4 themes, icons, and cursor successfully synchronized.");
        Ok(())
    }

    /// Convenience helper to initialize and apply the default Windows 11 Fluent dark theme
    pub fn apply_default() -> io::Result<ThemeConfig> {
        let service = Self::default();
        service.apply()?;
        Ok(service.config)
    }
}
