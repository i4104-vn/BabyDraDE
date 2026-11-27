use gio::DBusConnection;

use super::super::discovery::{connection_process_id, service_matches_app_id};
use super::connection::{
    accessible_children, connect_atspi, property_string, role_name, state_set,
};
use super::ids::{
    classify_role, state_has, AccessibleRef, AccessibleRole, A11Y_DESKTOP_PATH, A11Y_REGISTRY,
    MAX_TREE_NODES, MENU_TREE_DEPTH, STATE_ACTIVE, STATE_FOCUSED,
};

/// Finds the menu bar of the focused (or app-id/PID matching) accessibility
/// application. Returns `(accessibility bus name, menu bar object path)`.
pub(in crate::system::dbus_menu) fn find_atspi_menubar(
    app_id: &str,
    process_id: Option<u32>,
) -> Option<(String, String)> {
    let conn = connect_atspi()?;
    let desktop = AccessibleRef {
        bus_name: A11Y_REGISTRY.to_string(),
        object_path: A11Y_DESKTOP_PATH.to_string(),
    };
    let mut best: Option<(u8, AccessibleRef)> = None;

    for app in accessible_children(&conn, &desktop) {
        let app_pid = connection_process_id(&conn, &app.bus_name);
        if app_pid == Some(std::process::id()) {
            continue;
        }

        let name = property_string(&conn, &app, "Name").unwrap_or_default();
        let accessible_id = property_string(&conn, &app, "AccessibleId").unwrap_or_default();
        let pid_match = process_id.is_some_and(|pid| app_pid == Some(pid));
        let identity_match = !app_id.trim().is_empty()
            && (service_matches_app_id(&name, app_id)
                || service_matches_app_id(&accessible_id, app_id));
        let focused = has_focused_node(&conn, &app);
        let score = u8::from(pid_match) * 4 + u8::from(identity_match) * 2 + u8::from(focused);
        if score == 0 {
            continue;
        }

        let Some(menubar) = find_menu_bar(&conn, &app) else {
            continue;
        };
        if best
            .as_ref()
            .is_none_or(|(best_score, _)| score > *best_score)
        {
            best = Some((score, menubar));
        }
    }

    best.map(|(_, target)| (target.bus_name, target.object_path))
}

fn has_focused_node(conn: &DBusConnection, root: &AccessibleRef) -> bool {
    let mut pending = vec![(root.clone(), 0usize)];
    let mut visited = 0;
    while let Some((target, depth)) = pending.pop() {
        if visited >= MAX_TREE_NODES {
            break;
        }
        visited += 1;

        let states = state_set(conn, &target);
        if state_has(&states, STATE_ACTIVE) || state_has(&states, STATE_FOCUSED) {
            return true;
        }
        if depth < 2 {
            for child in accessible_children(conn, &target) {
                pending.push((child, depth + 1));
            }
        }
    }
    false
}

fn find_menu_bar(conn: &DBusConnection, app: &AccessibleRef) -> Option<AccessibleRef> {
    let mut pending = vec![(app.clone(), 0usize)];
    let mut visited = 0;
    let mut fallback = None;

    while let Some((target, depth)) = pending.pop() {
        if visited >= MAX_TREE_NODES {
            break;
        }
        visited += 1;

        let role = classify_role(&role_name(conn, &target));
        if matches!(role, AccessibleRole::MenuBar | AccessibleRole::Menu)
            && menu_bar_has_items(conn, &target)
        {
            if role == AccessibleRole::MenuBar {
                return Some(target);
            }
            if depth <= 3 && fallback.is_none() {
                fallback = Some(target.clone());
            }
        }

        if depth < MENU_TREE_DEPTH {
            for child in accessible_children(conn, &target) {
                pending.push((child, depth + 1));
            }
        }
    }

    fallback
}

fn menu_bar_has_items(conn: &DBusConnection, target: &AccessibleRef) -> bool {
    accessible_children(conn, target)
        .iter()
        .any(|child| classify_role(&role_name(conn, child)) != AccessibleRole::Other)
}
