//! Finding the D-Bus menu service exported by an application.

use super::discovery_helpers::{
    find_dbusmenu_path, find_registered_appmenus, get_connection_process_id,
    service_matches_app_id,
};
use super::fetch::fetch_dbus_menu_with_prepared_submenus;

/// Discovers the active application's D-Bus menu, preferring the window client's bus connection.
pub fn find_app_menu_service(app_id: &str, process_id: Option<u32>) -> Option<(String, String)> {
    if app_id.trim().is_empty() && process_id.is_none() {
        return None;
    }

    let conn = gio::bus_get_sync(gio::BusType::Session, gio::Cancellable::NONE).ok()?;

    const CANDIDATE_PATHS: &[&str] = &[
        "/MenuBar",
        "/MenuBar/1",
        "/com/canonical/menu",
        "/com/canonical/menu/1",
        "/com/canonical/AppMenu/topbar",
        "/org/chromium/DbusMenu",
        "/org/chromium/DbusMenu/1",
    ];

    let app_id_trimmed = app_id
        .trim()
        .strip_suffix(".desktop")
        .unwrap_or(app_id.trim());

    let reply = conn
        .call_sync(
            Some("org.freedesktop.DBus"),
            "/org/freedesktop/DBus",
            "org.freedesktop.DBus",
            "ListNames",
            None,
            None,
            gio::DBusCallFlags::NONE,
            400,
            gio::Cancellable::NONE,
        )
        .ok()?;

    if reply.n_children() == 0 {
        return None;
    }

    let names_var = reply.child_value(0);
    let n = names_var.n_children();
    let app_id_clean = app_id_trimmed.to_ascii_lowercase();
    let mut exact_name_candidates = Vec::new();
    let mut suffix_candidates = Vec::new();
    let mut process_candidates = Vec::new();
    let mut unique_names = Vec::new();

    for i in 0..n {
        let val = names_var.child_value(i);
        if let Some(name) = val.str() {
            if name.starts_with(':') {
                unique_names.push(name.to_string());
                continue;
            }
            let name_lower = name.to_ascii_lowercase();
            match true {
                _ if name_lower == app_id_clean => {
                    exact_name_candidates.push(name.to_string());
                }
                _ if name_lower.ends_with(&format!(".{app_id_clean}")) => {
                    suffix_candidates.push(name.to_string());
                }
                _ => {}
            }
        }
    }

    let registered_menus = find_registered_appmenus(&conn);

    if let Some(process_id) = process_id {
        process_candidates.extend(
            registered_menus
                .iter()
                .filter(|(service, _)| {
                    get_connection_process_id(&conn, service) == Some(process_id)
                })
                .cloned(),
        );

        // The focused Wayland client's PID is the strongest available association
        // between an xdg_toplevel and a D-Bus connection.
        for unique_name in unique_names {
            if get_connection_process_id(&conn, &unique_name) == Some(process_id) {
                process_candidates.push((unique_name, String::new()));
            }
        }
    }

    // Some toolkits export their menu from a helper process. In that case the
    // D-Bus PID differs from the Wayland client's PID, but its service still
    // commonly identifies the app.
    process_candidates.extend(
        registered_menus
            .into_iter()
            .filter(|(service, _)| service_matches_app_id(service, app_id_trimmed)),
    );

    let mut candidates = process_candidates;
    candidates.extend(
        exact_name_candidates
            .into_iter()
            .map(|service| (service, String::new())),
    );
    candidates.extend(
        suffix_candidates
            .into_iter()
            .map(|service| (service, String::new())),
    );
    let mut seen = std::collections::HashSet::new();

    for (service, registered_path) in candidates {
        if !seen.insert(service.clone()) && registered_path.is_empty() {
            continue;
        }

        if !registered_path.is_empty() && menu_has_visible_items(&service, &registered_path) {
            return Some((service, registered_path));
        }

        for path in CANDIDATE_PATHS {
            if menu_has_visible_items(&service, path) {
                return Some((service, path.to_string()));
            }
        }

        if let Some(path) = find_dbusmenu_path(&conn, &service) {
            if menu_has_visible_items(&service, &path) {
                return Some((service, path));
            }
        }
    }

    None
}

fn menu_has_visible_items(service: &str, path: &str) -> bool {
    fetch_dbus_menu_with_prepared_submenus(service, path).is_some_and(|items| !items.is_empty())
}
