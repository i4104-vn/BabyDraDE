//! Display-independent icon loading shared by every consumer.
//!
//! These helpers never require a GDK display, so they work in a bare TTY
//! compositor session as well as inside a GTK app.

use std::path::Path;

use gtk4::{
    gdk_pixbuf::Pixbuf,
    gio::{prelude::*, FileIcon, Icon, ThemedIcon},
};

/// Common icon-theme subdirectory names, largest first.
const SIZES: &[&str] = &[
    "scalable", "128x128", "96x96", "64x64", "48x48", "32x32", "24x24", "22x22", "16x16",
];

fn load_at(path: &str, size: i32) -> Option<Pixbuf> {
    if !Path::new(path).is_file() {
        return None;
    }
    Pixbuf::from_file_at_scale(path, size, size, true).ok()
}

/// Resolves an icon name (or absolute path) to a Pixbuf by scanning icon
/// theme directories on disk, without a GDK display.
pub fn get_icon_pixbuf_from_filesystem(name: &str, size: i32) -> Option<Pixbuf> {
    let path = Path::new(name);
    if path.is_absolute() {
        if let Some(pb) = load_at(name, size) {
            return Some(pb);
        }
    }

    let clean = name
        .strip_suffix(".png")
        .or_else(|| name.strip_suffix(".svg"))
        .unwrap_or(name);

    for base in services::theme::get_icon_search_paths() {
        let mut themes: Vec<String> = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&base) {
            for entry in entries.flatten() {
                if entry.path().join("index.theme").is_file() {
                    if let Some(name) = entry.file_name().to_str() {
                        themes.push(name.to_string());
                    }
                }
            }
        }
        // Prefer common themes, then whatever else is installed.
        themes.sort_by_key(|theme| {
            let priority = match theme.as_str() {
                "hicolor" => 0,
                "Adwaita" => 1,
                "Yaru" => 2,
                "breeze" => 3,
                "Papirus" => 4,
                _ => 5,
            };
            (priority, theme.clone())
        });

        for theme in themes {
            for sub in ["apps", "mimetypes", "devices", "actions"] {
                if let Some(pb) = theme_icon(&base, &theme, sub, clean, size) {
                    return Some(pb);
                }
            }
        }
    }

    for dir in [
        "/usr/share/pixmaps",
        "/usr/local/share/pixmaps",
        "/usr/share/icons/hicolor/scalable/apps",
    ] {
        for ext in ["png", "svg"] {
            if let Some(pb) = load_at(&format!("{dir}/{clean}.{ext}"), size) {
                return Some(pb);
            }
        }
    }

    None
}

fn theme_icon(base: &Path, theme: &str, sub: &str, name: &str, size: i32) -> Option<Pixbuf> {
    for s in SIZES {
        for ext in ["png", "svg"] {
            let candidate = base
                .join(theme)
                .join(s)
                .join(sub)
                .join(format!("{name}.{ext}"));
            if let Some(pb) = load_at(&candidate.to_string_lossy(), size) {
                return Some(pb);
            }
        }
    }
    None
}

/// Loads a Pixbuf from a `gio::Icon` without requiring a GDK display.
/// Handles file icons and themed icons (using the first available name).
pub fn pixbuf_from_gicon(icon: &Icon, size: i32) -> Option<Pixbuf> {
    if let Ok(file_icon) = icon.clone().downcast::<FileIcon>() {
        if let Some(path) = file_icon.file().path() {
            if let Some(pb) = load_at(&path.to_string_lossy(), size) {
                return Some(pb);
            }
        }
    }
    if let Ok(themed) = icon.clone().downcast::<ThemedIcon>() {
        for name in themed.names() {
            if let Some(pb) = get_icon_pixbuf_from_filesystem(name.as_ref(), size) {
                return Some(pb);
            }
        }
    }
    None
}
