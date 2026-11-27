//! Finding the application's menu bar across D-Bus and AT-SPI sources.

use std::collections::HashSet;

use super::atspi::{encode_atspi_source, find_atspi_menubar};
use super::discovery_helpers::{
    find_dbusmenu_paths, find_registered_appmenus, get_connection_process_id,
    service_matches_app_id,
};
use super::fetch::probe_dbus_menu;
use super::DbusMenuItem;

/// Standard locations used by GTK, Qt/KDE, Chromium and Electron app-menu
/// exporters. A service may use any one of them, so all are probed.
const CANDIDATE_PATHS: &[&str] = &[
    "/MenuBar",
    "/MenuBar/1",
    "/MenuBar/0",
    "/menu",
    "/menubar",
    "/dbusmenu",
    "/com/canonical/menu",
    "/com/canonical/menu/1",
    "/com/canonical/AppMenu/topbar",
    "/org/appmenu/menu",
    "/org/chromium/DbusMenu",
    "/org/chromium/DbusMenu/1",
    "/KMenuBar",
];
const MAX_SERVICES: usize = 18;
const MAX_PATHS: usize = 16;

#[derive(Debug, Clone)]
struct Candidate {
    service: String,
    /// Empty when the standard paths should be probed.
    registered_path: String,
    /// Lower is more trustworthy (registered menu before name guesses).
    priority: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MenuResult {
    score: usize,
    priority: u8,
}

impl MenuResult {
    fn is_better_than(self, other: Option<MenuResult>) -> bool {
        match other {
            Some(other) => {
                self.score > other.score
                    || (self.score == other.score && self.priority < other.priority)
            }
            None => true,
        }
    }
}

/// Discovers the active application's menu bar.
///
/// Returns a `(service, path)` pair. For AT-SPI sources `service` carries an
/// internal marker understood by [`super::fetch::fetch_app_menu`], so callers
/// do not need a different menu handle type.
pub fn find_app_menu_service(app_id: &str, process_id: Option<u32>) -> Option<(String, String)> {
    if app_id.trim().is_empty() && process_id.is_none() {
        return None;
    }

    let conn = gio::bus_get_sync(gio::BusType::Session, gio::Cancellable::NONE).ok()?;
    let names = list_bus_names(&conn);
    let registered = find_registered_appmenus(&conn);
    let candidates = build_candidates(&names, &registered, app_id, process_id, &conn);

    let mut best: Option<MenuResult> = None;
    let mut best_service = String::new();
    let mut best_path = String::new();

    for candidate in candidates.iter().take(MAX_SERVICES) {
        for path in candidate_paths(candidate) {
            let Some(items) = probe_dbus_menu(&candidate.service, path) else {
                continue;
            };
            let score = menu_quality(&items);
            if score == 0 {
                continue;
            }
            let result = MenuResult {
                score,
                priority: candidate.priority,
            };
            if result.is_better_than(best) {
                best = Some(result);
                best_service.clone_from(&candidate.service);
                best_path = (*path).to_string();
            }
        }
    }

    if best.is_some() {
        return Some((best_service, best_path));
    }

    // Some exporters only install their object at an unexpected path. Ask the
    // service to introspect its tree instead of relying on the hardcoded list.
    for candidate in candidates.iter().take(MAX_SERVICES) {
        for (path, _depth) in find_dbusmenu_paths(&conn, &candidate.service) {
            let Some(items) = probe_dbus_menu(&candidate.service, &path) else {
                continue;
            };
            let score = menu_quality(&items);
            if score == 0 {
                continue;
            }
            let result = MenuResult {
                score,
                priority: candidate.priority.saturating_add(4),
            };
            if result.is_better_than(best) {
                best = Some(result);
                best_service.clone_from(&candidate.service);
                best_path = path;
            }
        }
    }

    if best.is_some() {
        return Some((best_service, best_path));
    }

    // Electron/VS Code and Chromium usually have no dbusmenu exporter at all.
    find_atspi_menubar(app_id, process_id)
        .map(|(bus_name, path)| encode_atspi_source(&bus_name, &path))
}

fn list_bus_names(conn: &gio::DBusConnection) -> Vec<String> {
    let reply = match conn.call_sync(
        Some("org.freedesktop.DBus"),
        "/org/freedesktop/DBus",
        "org.freedesktop.DBus",
        "ListNames",
        None,
        None,
        gio::DBusCallFlags::NONE,
        400,
        gio::Cancellable::NONE,
    ) {
        Ok(reply) => reply,
        Err(_) => return Vec::new(),
    };
    let names = reply.child_value(0);
    (0..names.n_children())
        .filter_map(|index| names.child_value(index).str().map(str::to_string))
        .collect()
}

fn build_candidates(
    names: &[String],
    registered: &[(String, String)],
    app_id: &str,
    process_id: Option<u32>,
    conn: &gio::DBusConnection,
) -> Vec<Candidate> {
    let mut candidates = Vec::new();
    let mut seen = HashSet::new();

    let mut push = |service: String, path: String, priority: u8| {
        if service.is_empty() {
            return;
        }
        let key = format!("{service}\0{path}");
        if seen.insert(key) {
            candidates.push(Candidate {
                service,
                registered_path: path,
                priority,
            });
        }
    };

    // Registrar entries are authoritative: they were published by the app.
    // Unrelated registrations are excluded so an empty/wrong menu cannot win.
    for (service, path) in registered {
        let belongs_to_app = service_matches_app_id(service, app_id)
            || process_id.is_some_and(|pid| get_connection_process_id(conn, service) == Some(pid));
        if belongs_to_app {
            push(service.clone(), path.clone(), 0);
        }
    }

    for name in names {
        if name.starts_with(':') {
            continue;
        }
        if service_matches_app_id(name, app_id) {
            push(name.clone(), String::new(), 1);
        }
    }

    // KDE's application-menu aggregator exports the focused window's menubar.
    // It is a provider, not an application name, and is used as a last resort.
    for name in names {
        let lower = name.to_ascii_lowercase();
        if lower == "org.kde.kappmenu"
            || lower.starts_with("org.kde.kappmenu.")
            || lower == "org.kde.appmenu"
            || lower.starts_with("org.kde.appmenu.")
        {
            push(name.clone(), String::new(), 3);
        }
    }

    if let Some(pid) = process_id {
        for name in names {
            if get_connection_process_id(conn, name) == Some(pid) {
                let priority = if name.starts_with(':') { 2 } else { 1 };
                push(name.clone(), String::new(), priority);
            }
        }
    }

    candidates
}

fn candidate_paths(candidate: &Candidate) -> Vec<&str> {
    let mut paths: Vec<&str> = Vec::with_capacity(CANDIDATE_PATHS.len() + 1);
    if !candidate.registered_path.is_empty() {
        paths.push(&candidate.registered_path);
    }
    paths.extend_from_slice(CANDIDATE_PATHS);
    paths.truncate(MAX_PATHS);
    paths
}

/// Scores a menu by the number of usable entries it exposes. A candidate that
/// merely returns separators or nameless entries is rejected, so an unrelated
/// exporter with an empty/inert menu cannot win the race.
pub(super) fn menu_quality(items: &[DbusMenuItem]) -> usize {
    items
        .iter()
        .filter(|item| item.visible && !item.is_separator && !item.label.trim().is_empty())
        .map(|item| 1 + item.children.len().min(6))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::menu_quality;
    use crate::system::dbus_menu::DbusMenuItem;

    #[test]
    fn menu_quality_rejects_empty_and_separator_only_menus() {
        assert_eq!(menu_quality(&[]), 0);
        let separators = vec![DbusMenuItem {
            visible: true,
            is_separator: true,
            ..Default::default()
        }];
        assert_eq!(menu_quality(&separators), 0);
    }

    #[test]
    fn menu_quality_prefers_richer_menus() {
        let small = vec![DbusMenuItem {
            label: "File".to_string(),
            visible: true,
            ..Default::default()
        }];
        let rich = vec![
            DbusMenuItem {
                label: "File".to_string(),
                visible: true,
                children: vec![DbusMenuItem::default(); 8],
                ..Default::default()
            },
            DbusMenuItem {
                label: "Edit".to_string(),
                visible: true,
                ..Default::default()
            },
        ];
        assert!(menu_quality(&rich) > menu_quality(&small));
        assert_eq!(
            menu_quality(&[DbusMenuItem {
                label: "Hidden".to_string(),
                visible: false,
                ..Default::default()
            }]),
            0
        );
    }
}
