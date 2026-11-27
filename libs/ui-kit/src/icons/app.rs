//! app icon helpers.

use super::*;

fn clean_app_id(app_id: &str) -> &str {
    let trimmed = app_id.trim();
    trimmed.strip_suffix(".desktop").unwrap_or(trimmed)
}

/// Case/punctuation-insensitive form used to compare desktop entry names.
fn normalise(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .collect()
}

/// Finds an installed desktop entry's icon for fallback candidates.
///
/// Candidates arrive ordered by confidence from the window identity helper;
/// this path is used only when no pid-backed entry was identified.
fn gicon_for_candidates(candidates: &[String]) -> Option<gtk4::gio::Icon> {
    if candidates.is_empty() {
        return None;
    }
    let infos = gtk4::gio::AppInfo::all();
    for candidate in candidates {
        let clean = clean_app_id(candidate);
        let needle = normalise(candidate);
        for info in &infos {
            let id_matches = info
                .id()
                .is_some_and(|id| clean_app_id(&id).eq_ignore_ascii_case(clean));
            let name_matches = !needle.is_empty() && normalise(&info.name()) == needle;
            if !id_matches && !name_matches {
                continue;
            }
            if let Some(icon) = info.icon() {
                return Some(icon);
            }
        }
    }
    None
}

fn icon_name_from_gicon(icon: gtk4::gio::Icon) -> Option<String> {
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
    icon.to_string()
        .filter(|name| !name.is_empty())
        .map(|name| name.to_string())
}

/// Takes the icon of one exact installed desktop entry, never a name-derived
/// substitute. The id comes from the same PID-based resolution as the label.
fn gicon_for_entry_id(entry_id: &str) -> Option<gtk4::gio::Icon> {
    let clean = clean_app_id(entry_id);
    gtk4::gio::AppInfo::all()
        .into_iter()
        .find(|info| {
            info.id()
                .is_some_and(|id| clean_app_id(&id).eq_ignore_ascii_case(clean))
        })
        .and_then(|info| info.icon())
}

/// Resolves a window's icon from the same desktop entry that supplied its name.
///
/// With a pid, the entry belongs to the real process; title candidates are only
/// consulted when no entry could be identified. Without a pid this is the
/// historical app-id/title fallback.
fn gicon_for_window_pid(
    app_id: &str,
    title: &str,
    process_id: Option<u32>,
) -> Option<gtk4::gio::Icon> {
    match services::system::window::get_window_desktop_entry_id(app_id, title, process_id) {
        Some(entry_id) => gicon_for_entry_id(&entry_id),
        None => {
            let candidates =
                services::system::window::get_window_app_candidates(app_id, title, process_id);
            gicon_for_candidates(&candidates)
        }
    }
}

/// Resolves a desktop entry icon for a window, using the title only when no
/// pid/entry could identify the application.
pub fn resolve_app_gicon_for_window(app_id: &str, title: &str) -> Option<gtk4::gio::Icon> {
    resolve_app_gicon_for_window_pid(app_id, title, None)
}

/// PID-aware variant of [`resolve_app_gicon_for_window`]. The label and the icon
/// are resolved from the very same installed desktop entry.
pub fn resolve_app_gicon_for_window_pid(
    app_id: &str,
    title: &str,
    process_id: Option<u32>,
) -> Option<gtk4::gio::Icon> {
    gicon_for_window_pid(app_id, title, process_id)
}

/// Resolves an app_id to its icon GIcon using desktop files.
pub fn resolve_app_gicon(app_id: &str) -> Option<gtk4::gio::Icon> {
    resolve_app_gicon_for_window(app_id, "")
}

/// Resolves a window's icon name, using its title only as a fallback.
pub fn resolve_app_icon_name_for_window(app_id: &str, title: &str) -> Option<String> {
    resolve_app_icon_name_for_window_pid(app_id, title, None)
}

/// PID-aware variant of [`resolve_app_icon_name_for_window`].
pub fn resolve_app_icon_name_for_window_pid(
    app_id: &str,
    title: &str,
    process_id: Option<u32>,
) -> Option<String> {
    let candidates = services::system::window::get_window_app_candidates(app_id, title, process_id);
    if let Some(icon) = gicon_for_window_pid(app_id, title, process_id) {
        if let Some(name) = icon_name_from_gicon(icon) {
            return Some(name);
        }
    }
    candidates.first().cloned().or_else(|| {
        let clean = clean_app_id(app_id);
        (!clean.is_empty()).then(|| clean.to_string())
    })
}

/// Resolves an app_id to its icon name using gio::AppInfo desktop files.
pub fn resolve_app_icon_name(app_id: &str) -> Option<String> {
    resolve_app_icon_name_for_window(app_id, "")
}

/// Retrieves a themed Pixbuf for a window, disambiguating shared app ids with
/// the window title when no pid is available.
pub fn get_app_icon_pixbuf_for_window(app_id: &str, title: &str, size: i32) -> Option<Pixbuf> {
    get_app_icon_pixbuf_for_window_pid(app_id, title, size, None)
}

/// PID-aware variant of [`get_app_icon_pixbuf_for_window`].
pub fn get_app_icon_pixbuf_for_window_pid(
    app_id: &str,
    title: &str,
    size: i32,
    process_id: Option<u32>,
) -> Option<Pixbuf> {
    let icon_name = resolve_app_icon_name_for_window_pid(app_id, title, process_id)?;
    get_theme_icon_pixbuf(&icon_name, size)
}

/// Retrieves a Pixbuf for an app by app_id, looking up via AppInfo and GtkIconTheme.
pub fn get_app_icon_pixbuf(app_id: &str, size: i32) -> Option<Pixbuf> {
    get_app_icon_pixbuf_for_window(app_id, "", size)
}

/// Display-independent app icon lookup.
///
/// Resolution order: installed desktop entry matched by observed data → on-disk
/// icon theme scan → display-backed GtkIconTheme → `None`. Never panics.
pub fn get_app_icon_pixbuf_any_for_window(app_id: &str, title: &str, size: i32) -> Option<Pixbuf> {
    get_app_icon_pixbuf_any_for_window_pid(app_id, title, size, None)
}

/// PID-aware variant of [`get_app_icon_pixbuf_any_for_window`]. Uses the exact
/// icon of the desktop entry belonging to the window's real process.
pub fn get_app_icon_pixbuf_any_for_window_pid(
    app_id: &str,
    title: &str,
    size: i32,
    process_id: Option<u32>,
) -> Option<Pixbuf> {
    let candidates = services::system::window::get_window_app_candidates(app_id, title, process_id);
    if let Some(icon) = gicon_for_window_pid(app_id, title, process_id) {
        if let Some(pb) = pixbuf_from_gicon(&icon, size) {
            return Some(pb);
        }
    }
    for candidate in &candidates {
        if let Some(pb) = get_icon_pixbuf_from_filesystem(candidate, size) {
            return Some(pb);
        }
        if let Some(pb) = get_theme_icon_pixbuf(candidate, size) {
            return Some(pb);
        }
    }
    let clean = clean_app_id(app_id);
    if !clean.is_empty() {
        return get_icon_pixbuf_from_filesystem(clean, size)
            .or_else(|| get_theme_icon_pixbuf(clean, size));
    }
    None
}

/// Single entry point for app-id → Pixbuf that works everywhere, including a
/// bare compositor session with no GDK display.
///
/// Resolution order: AppInfo GIcon (file/themed) → on-disk icon theme scan →
/// display-backed GtkIconTheme → `None`. Never panics.
pub fn get_app_icon_pixbuf_any(app_id: &str, size: i32) -> Option<Pixbuf> {
    get_app_icon_pixbuf_any_for_window(app_id, "", size)
}
