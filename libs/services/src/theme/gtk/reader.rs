use std::fs;
use std::path::PathBuf;

use crate::theme::model::GtkThemeConfig;

pub fn find_settings_ini_path() -> Option<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
        let p = PathBuf::from(xdg_config);
        candidates.push(p.join("gtk-4.0").join("settings.ini"));
        candidates.push(p.join("gtk-3.0").join("settings.ini"));
    }

    if let Ok(home) = std::env::var("HOME") {
        let p = PathBuf::from(home).join(".config");
        candidates.push(p.join("gtk-4.0").join("settings.ini"));
        candidates.push(p.join("gtk-3.0").join("settings.ini"));
    }

    candidates.push(PathBuf::from("/etc/gtk-4.0/settings.ini"));
    candidates.push(PathBuf::from("/etc/gtk-3.0/settings.ini"));

    candidates.into_iter().find(|p| p.is_file())
}

pub fn read_gtk_settings() -> GtkThemeConfig {
    let path = match find_settings_ini_path() {
        Some(p) => p,
        None => return GtkThemeConfig::default(),
    };

    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return GtkThemeConfig::default(),
    };

    parse_settings_ini(&content, Some(path.to_string_lossy().to_string()))
}

pub fn parse_settings_ini(content: &str, source_path: Option<String>) -> GtkThemeConfig {
    let mut config = GtkThemeConfig {
        config_source_path: source_path,
        ..Default::default()
    };

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.starts_with(';') || trimmed.starts_with('[') {
            continue;
        }

        let mut parts = trimmed.splitn(2, '=');
        let key = match parts.next() {
            Some(k) => k.trim(),
            None => continue,
        };
        let val = match parts.next() {
            Some(v) => v.trim().trim_matches('"').trim_matches('\''),
            None => continue,
        };

        match key {
            "gtk-theme-name" => {
                if !val.is_empty() {
                    config.theme_name = val.to_string();
                }
            }
            "gtk-icon-theme-name" => {
                if !val.is_empty() {
                    config.icon_theme = val.to_string();
                }
            }
            "gtk-cursor-theme-name" => {
                if !val.is_empty() {
                    config.cursor_theme = val.to_string();
                }
            }
            "gtk-cursor-theme-size" => {
                if let Ok(size) = val.parse::<u32>() {
                    config.cursor_size = size;
                }
            }
            "gtk-font-name" => {
                if !val.is_empty() {
                    config.font_name = val.to_string();
                }
            }
            "gtk-application-prefer-dark-theme" => {
                config.prefer_dark_theme = val == "1" || val.eq_ignore_ascii_case("true");
            }
            "gtk-decoration-layout" => {
                config.decoration_layout = Some(val.to_string());
            }
            "gtk-enable-animations" => {
                config.enable_animations = val == "1" || val.eq_ignore_ascii_case("true");
            }
            _ => {}
        }
    }

    config
}
