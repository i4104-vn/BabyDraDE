//! Central Theme Engine coordinator
//!
//! Manages GTK4 styling, detects Dark/Light mode preference from GTK configuration,
//! and dynamically builds and hot-reloads stylesheets across all widgets.

use std::cell::Cell;
use std::path::PathBuf;

use gtk4::gdk::Display;
use gtk4::{CssProvider, STYLE_PROVIDER_PRIORITY_USER};

use super::dark_css::DARK_CSS;
use super::light_css::LIGHT_CSS;
use super::shared_css::SHARED_CSS;

thread_local! {
    static CSS_PROVIDER: CssProvider = CssProvider::new();
    static REGISTERED: Cell<bool> = const { Cell::new(false) };
}

/// Attempts to read the dark theme preference from GTK configuration on disk.
fn read_gtk_prefer_dark_from_disk() -> Option<bool> {
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

    for path in candidates {
        if let Ok(content) = std::fs::read_to_string(&path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with('#') || trimmed.starts_with(';') {
                    continue;
                }
                let mut parts = trimmed.splitn(2, '=');
                if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
                    let key = k.trim();
                    let val = v.trim().trim_matches('"').trim_matches('\'');
                    if key.eq_ignore_ascii_case("gtk-application-prefer-dark-theme") {
                        return Some(val == "1" || val.eq_ignore_ascii_case("true"));
                    }
                }
            }
        }
    }

    None
}

/// Checks whether Dark mode is currently active based on GTK settings.
pub fn is_dark_mode() -> bool {
    if let Some(settings) = gtk4::Settings::default() {
        return settings.is_gtk_application_prefer_dark_theme();
    }
    read_gtk_prefer_dark_from_disk().unwrap_or(true)
}

/// Toggles or sets the dark mode preference in GTK settings.
pub fn set_dark_mode(dark: bool) {
    if let Some(settings) = gtk4::Settings::default() {
        settings.set_gtk_application_prefer_dark_theme(dark);
    }
    reload_theme();
}

/// Builds the combined CSS stylesheet string for the requested dark/light mode.
pub fn build_css(is_dark: bool) -> String {
    let color_layer = if is_dark { DARK_CSS } else { LIGHT_CSS };
    format!("{SHARED_CSS}\n{color_layer}")
}

/// Returns the full active CSS stylesheet string for the current mode.
pub fn get_active_css() -> String {
    build_css(is_dark_mode())
}

/// Forces a theme reload using the current GTK settings.
pub fn reload_theme() {
    CSS_PROVIDER.with(|provider| {
        let is_dark = is_dark_mode();
        let css = build_css(is_dark);
        provider.load_from_data(&css);
    });
}

/// Initializes the GtkCssProvider, registers it with GdkDisplay,
/// hooks GTK dark theme notifications for live hot-reloading,
/// and immediately applies the appropriate CSS.
pub fn init_theme() {
    // 1. Sync settings if GTK default has not been loaded from disk
    if let Some(settings) = gtk4::Settings::default() {
        if let Some(disk_dark) = read_gtk_prefer_dark_from_disk() {
            settings.set_gtk_application_prefer_dark_theme(disk_dark);
        }
    }

    CSS_PROVIDER.with(|provider| {
        let is_registered = REGISTERED.with(|r| r.get());
        if !is_registered {
            if let Some(display) = Display::default() {
                gtk4::style_context_add_provider_for_display(
                    &display,
                    provider,
                    STYLE_PROVIDER_PRIORITY_USER,
                );
            }

            // Listen for dark/light preference notifications
            if let Some(settings) = gtk4::Settings::default() {
                let provider_clone = provider.clone();
                settings.connect_gtk_application_prefer_dark_theme_notify(move |s| {
                    let is_dark = s.is_gtk_application_prefer_dark_theme();
                    let css = build_css(is_dark);
                    provider_clone.load_from_data(&css);
                });
            }

            REGISTERED.with(|r| r.set(true));
        }

        let css = build_css(is_dark_mode());
        provider.load_from_data(&css);
    });
}
