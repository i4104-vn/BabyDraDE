#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorScheme {
    Dark,
    Light,
    Default,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self::Dark
    }
}

impl ColorScheme {
    pub fn as_gsettings_str(&self) -> &'static str {
        match self {
            Self::Dark => "prefer-dark",
            Self::Light => "prefer-light",
            Self::Default => "default",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeConfig {
    pub gtk_theme: String,
    pub icon_theme: String,
    pub cursor_theme: String,
    pub cursor_size: u32,
    pub font_name: String,
    pub color_scheme: ColorScheme,
    pub enable_animations: bool,
    pub custom_css: Option<String>,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self::windows11_fluent_dark()
    }
}

impl ThemeConfig {
    /// Preset: Windows 11 Fluent Dark Look & Feel
    pub fn windows11_fluent_dark() -> Self {
        Self {
            gtk_theme: "Adwaita-dark".to_string(),
            icon_theme: "Fluent-dark".to_string(),
            cursor_theme: "Adwaita".to_string(),
            cursor_size: 24,
            font_name: "Segoe UI Variable 11".to_string(),
            color_scheme: ColorScheme::Dark,
            enable_animations: true,
            custom_css: Some(Self::default_fluent_css()),
        }
    }

    /// Preset: Windows 11 Fluent Light Look & Feel
    pub fn windows11_fluent_light() -> Self {
        Self {
            gtk_theme: "Adwaita".to_string(),
            icon_theme: "Fluent".to_string(),
            cursor_theme: "Adwaita".to_string(),
            cursor_size: 24,
            font_name: "Segoe UI Variable 11".to_string(),
            color_scheme: ColorScheme::Light,
            enable_animations: true,
            custom_css: None,
        }
    }

    pub fn default_fluent_css() -> String {
        r#"/* BabyDraDE Fluent Dark Style for GTK4 */
@define-color accent_color #00a4ef;
@define-color accent_bg_color #00a4ef;
@define-color accent_fg_color #ffffff;
@define-color window_bg_color #202020;
@define-color window_fg_color #f3f3f3;
@define-color headerbar_bg_color #2b2b2b;
@define-color headerbar_fg_color #f3f3f3;

window.background {
    border-radius: 8px;
}

headerbar {
    border-radius: 8px 8px 0 0;
    min-height: 38px;
    padding: 0 6px;
}

/* Rounded Buttons */
button {
    border-radius: 4px;
}

/* Subtle scrollbars */
scrollbar {
    background-color: transparent;
}
scrollbar slider {
    min-width: 6px;
    min-height: 6px;
    border-radius: 3px;
}
"#
        .to_string()
    }
}
