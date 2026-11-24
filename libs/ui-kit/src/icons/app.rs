//! app icon helpers.

use super::*;

pub fn resolve_app_gicon(app_id: &str) -> Option<gtk4::gio::Icon> {
    let clean_id = app_id
        .trim()
        .strip_suffix(".desktop")
        .unwrap_or(app_id.trim());
    if clean_id.is_empty() {
        return None;
    }
    gtk4::gio::AppInfo::all()
        .into_iter()
        .find(|info| {
            info.id().is_some_and(|id| {
                id.strip_suffix(".desktop")
                    .unwrap_or(id.as_str())
                    .eq_ignore_ascii_case(clean_id)
            })
        })
        .and_then(|info| info.icon())
}

/// Resolves an app_id to its icon name using gio::AppInfo desktop files.
pub fn resolve_app_icon_name(app_id: &str) -> Option<String> {
    let clean_id = app_id
        .trim()
        .strip_suffix(".desktop")
        .unwrap_or(app_id.trim());

    if clean_id.is_empty() {
        return None;
    }

    for info in gtk4::gio::AppInfo::all() {
        if let Some(id) = info.id() {
            let id_clean = id.strip_suffix(".desktop").unwrap_or(id.as_str());
            if id_clean.eq_ignore_ascii_case(clean_id) {
                if let Some(icon) = info.icon() {
                    if let Ok(themed) = icon.clone().downcast::<gtk4::gio::ThemedIcon>() {
                        if let Some(first) = themed.names().first() {
                            return Some(first.to_string());
                        }
                    }
                    if let Ok(file_icon) = icon.clone().downcast::<gtk4::gio::FileIcon>() {
                        if let Some(path) = file_icon.file().path() {
                            return Some(path.to_string_lossy().to_string());
                        }
                    }
                    if let Some(to_str) = icon.to_string() {
                        if !to_str.is_empty() {
                            return Some(to_str.to_string());
                        }
                    }
                }
                break;
            }
        }
    }

    Some(clean_id.to_string())
}

/// Retrieves a Pixbuf for an app by app_id, looking up via AppInfo and GtkIconTheme.
pub fn get_app_icon_pixbuf(app_id: &str, size: i32) -> Option<Pixbuf> {
    let icon_name = resolve_app_icon_name(app_id)?;
    get_theme_icon_pixbuf(&icon_name, size)
}

/// Single entry point for app-id → Pixbuf that works everywhere, including a
/// bare compositor session with no GDK display.
///
/// Resolution order: AppInfo GIcon (file/themed) → on-disk icon theme scan →
/// display-backed GtkIconTheme → `None`. Never panics.
pub fn get_app_icon_pixbuf_any(app_id: &str, size: i32) -> Option<Pixbuf> {
    let clean_id = app_id
        .trim()
        .strip_suffix(".desktop")
        .unwrap_or(app_id.trim());

    let info = gtk4::gio::AppInfo::all().into_iter().find(|info| {
        info.id().is_some_and(|id| {
            id.strip_suffix(".desktop")
                .unwrap_or(id.as_str())
                .eq_ignore_ascii_case(clean_id)
        })
    });
    if let Some(info) = info {
        if let Some(icon) = info.icon() {
            if let Some(pb) = pixbuf_from_gicon(&icon, size) {
                return Some(pb);
            }
        }
    }
    get_icon_pixbuf_from_filesystem(clean_id, size)
        .or_else(|| get_theme_icon_pixbuf(clean_id, size))
}
