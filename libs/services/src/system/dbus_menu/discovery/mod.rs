//! Finding the application's menu bar across D-Bus and AT-SPI sources.

mod app_id;
mod bus_names;
mod candidates;
mod paths;
mod quality;

use super::atspi::{encode_atspi_source, find_atspi_menubar};
use super::fetch::probe_dbus_menu;
use bus_names::{find_registered_appmenus, list_bus_names};
use candidates::{build_candidates, candidate_paths, MenuResult, MAX_SERVICES};
use paths::find_dbusmenu_paths;

pub(super) use app_id::{connection_process_id, service_matches_app_id};
pub(super) use paths::variant_string;
pub(super) use quality::menu_quality;

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
                best_path = path.to_string();
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
