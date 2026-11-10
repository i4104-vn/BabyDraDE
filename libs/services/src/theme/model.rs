#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GtkThemeConfig {
    pub theme_name: String,
    pub icon_theme: String,
    pub cursor_theme: String,
    pub cursor_size: u32,
    pub font_name: String,
    pub prefer_dark_theme: bool,
    pub decoration_layout: Option<String>,
    pub enable_animations: bool,
    pub config_source_path: Option<String>,
}

impl Default for GtkThemeConfig {
    fn default() -> Self {
        Self {
            theme_name: "Adwaita".to_string(),
            icon_theme: "Adwaita".to_string(),
            cursor_theme: "Adwaita".to_string(),
            cursor_size: 24,
            font_name: "Sans 11".to_string(),
            prefer_dark_theme: true,
            decoration_layout: Some(":minimize,maximize,close".to_string()),
            enable_animations: true,
            config_source_path: None,
        }
    }
}

/// Canvas clear color theme for compositor backends.
#[derive(Debug, Clone, Copy)]
pub struct DesktopTheme {
    /// Canvas clear color [R, G, B, A]
    pub background_color: [f32; 4],
}

impl Default for DesktopTheme {
    fn default() -> Self {
        Self {
            background_color: [0.05, 0.05, 0.07, 1.0],
        }
    }
}

