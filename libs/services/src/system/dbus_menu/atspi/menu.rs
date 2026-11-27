use std::collections::HashSet;

use gio::DBusConnection;

use super::super::parse::normalize_menu_label;
use super::super::{DbusMenuItem, MenuActionRef};
use super::connection::{
    accessible_children, connect_atspi, has_action, property_string, role_name, state_set,
};
use super::ids::{
    classify_role, state_has, AccessibleRef, AccessibleRole, MAX_TREE_NODES, MENU_TREE_DEPTH,
    STATE_DEFUNCT, STATE_HAS_POPUP, STATE_SENSITIVE,
};

/// Reads the full menu tree for an AT-SPI menu bar.
pub(in crate::system::dbus_menu) fn fetch_atspi_menu(
    bus_name: &str,
    menu_path: &str,
) -> Option<Vec<DbusMenuItem>> {
    let conn = connect_atspi()?;
    let root = AccessibleRef {
        bus_name: bus_name.to_string(),
        object_path: menu_path.to_string(),
    };
    let mut context = BuildContext::default();
    let items = build_menu_items(&conn, &root, 0, &mut context);
    (!items.is_empty()).then_some(items)
}

#[derive(Default)]
struct BuildContext {
    next_id: i32,
    visited: HashSet<String>,
}

fn build_menu_items(
    conn: &DBusConnection,
    container: &AccessibleRef,
    depth: usize,
    context: &mut BuildContext,
) -> Vec<DbusMenuItem> {
    if depth > MENU_TREE_DEPTH || context.visited.len() >= MAX_TREE_NODES {
        return Vec::new();
    }

    let mut items = Vec::new();
    for target in accessible_children(conn, container) {
        if context.visited.len() >= MAX_TREE_NODES {
            break;
        }
        let key = format!("{}:{}", target.bus_name, target.object_path);
        if !context.visited.insert(key) {
            continue;
        }

        let role = classify_role(&role_name(conn, &target));
        if role == AccessibleRole::Other {
            continue;
        }

        let states = state_set(conn, &target);
        let label =
            normalize_menu_label(&property_string(conn, &target, "Name").unwrap_or_default());
        let children = if role == AccessibleRole::Separator {
            Vec::new()
        } else {
            build_menu_items(conn, &target, depth + 1, context)
        };
        if label.is_empty() && role != AccessibleRole::Separator && children.is_empty() {
            continue;
        }

        // `target` is owned by this loop iteration and is not used afterwards,
        // so move its fields into the action instead of cloning them.
        let action = if has_action(conn, &target) {
            Some(MenuActionRef {
                bus_name: target.bus_name,
                object_path: target.object_path,
            })
        } else {
            None
        };
        let id = context.next_id;
        context.next_id += 1;

        items.push(DbusMenuItem {
            id,
            label,
            enabled: role == AccessibleRole::Separator
                || states.is_empty()
                || state_has(&states, STATE_SENSITIVE),
            visible: !state_has(&states, STATE_DEFUNCT),
            is_separator: role == AccessibleRole::Separator,
            has_submenu: !children.is_empty() || state_has(&states, STATE_HAS_POPUP),
            toggle_type: None,
            toggle_state: None,
            children,
            action,
        });
    }
    items
}
