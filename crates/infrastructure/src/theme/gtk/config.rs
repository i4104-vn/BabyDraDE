use std::fs;
use std::io;
use std::path::PathBuf;

use crate::theme::model::ThemeConfig;

pub fn get_config_dir() -> PathBuf {
    if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
        let p = PathBuf::from(xdg_config);
        if !p.as_os_str().is_empty() {
            return p;
        }
    }

    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".config");
    }

    PathBuf::from("/tmp/.config")
}

pub fn generate_settings_ini(config: &ThemeConfig) -> String {
    let prefer_dark = match config.color_scheme {
        crate::theme::model::ColorScheme::Dark => "1",
        _ => "0",
    };

    let animations = if config.enable_animations {
        "true"
    } else {
        "false"
    };

    format!(
        r#"[Settings]
gtk-theme-name = {gtk_theme}
gtk-icon-theme-name = {icon_theme}
gtk-cursor-theme-name = {cursor_theme}
gtk-cursor-theme-size = {cursor_size}
gtk-font-name = {font_name}
gtk-application-prefer-dark-theme = {prefer_dark}
gtk-decoration-layout = :minimize,maximize,close
gtk-enable-animations = {animations}
gtk-primary-button-warps-slider = false
"#,
        gtk_theme = config.gtk_theme,
        icon_theme = config.icon_theme,
        cursor_theme = config.cursor_theme,
        cursor_size = config.cursor_size,
        font_name = config.font_name,
        prefer_dark = prefer_dark,
        animations = animations,
    )
}

pub fn apply_gtk_settings(config: &ThemeConfig) -> io::Result<()> {
    let config_dir = get_config_dir();
    let content = generate_settings_ini(config);

    // 1. Write GTK-4.0 settings
    let gtk4_dir = config_dir.join("gtk-4.0");
    fs::create_dir_all(&gtk4_dir)?;
    fs::write(gtk4_dir.join("settings.ini"), &content)?;

    // 2. Write GTK-3.0 settings (for legacy GTK3 apps)
    let gtk3_dir = config_dir.join("gtk-3.0");
    fs::create_dir_all(&gtk3_dir)?;
    fs::write(gtk3_dir.join("settings.ini"), &content)?;

    Ok(())
}

pub fn read_current_gtk4_settings() -> Option<String> {
    let path = get_config_dir().join("gtk-4.0").join("settings.ini");
    fs::read_to_string(path).ok()
}
