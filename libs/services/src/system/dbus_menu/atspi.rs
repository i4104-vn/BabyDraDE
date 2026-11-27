//! AT-SPI accessibility menu source.
//!
//! Electron (VS Code and its forks), Chromium and some Qt/GTK applications do
//! not export `com.canonical.dbusmenu`: their menu bar is only reachable via
//! the accessibility bus (`org.a11y.atspi.*`). This module speaks that D-Bus
//! API directly, so no additional crate or workspace manifest change is
//! required. It is used as a fallback after D-Bus menu discovery fails.

use std::collections::HashSet;

use gio::glib::Variant;
use gio::DBusConnection;

use super::discovery_helpers::{connection_process_id, service_matches_app_id, variant_string};
use super::models::{DbusMenuItem, MenuActionRef};
use super::parse::normalize_menu_label;

const A11Y_BUS_SERVICE: &str = "org.a11y.Bus";
const A11Y_BUS_PATH: &str = "/org/a11y/bus";
const A11Y_REGISTRY: &str = "org.a11y.atspi.Registry";
const A11Y_DESKTOP_PATH: &str = "/org/a11y/atspi/accessible/root";
const ACCESSIBLE_IFACE: &str = "org.a11y.atspi.Accessible";
const ACTION_IFACE: &str = "org.a11y.atspi.Action";
const PROPERTIES_IFACE: &str = "org.freedesktop.DBus.Properties";

const STATE_ACTIVE: usize = 1;
const STATE_DEFUNCT: usize = 6;
const STATE_FOCUSED: usize = 12;
const STATE_SENSITIVE: usize = 24;
const STATE_HAS_POPUP: usize = 42;

const CALL_TIMEOUT_MS: i32 = 150;
const MAX_TREE_NODES: usize = 160;
const MENU_TREE_DEPTH: usize = 6;

/// Marker used in the `(service, path)` pair returned by discovery so the
/// topbar can keep using the existing string-based menu handle.
const SOURCE_PREFIX: &str = "atspi:";

#[derive(Debug, Clone, PartialEq, Eq)]
struct AccessibleRef {
    bus_name: String,
    object_path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AccessibleRole {
    MenuBar,
    Menu,
    MenuItem,
    Separator,
    Other,
}

/// Encodes an AT-SPI target into the service/path pair used by the UI.
pub(super) fn encode_atspi_source(bus_name: &str, object_path: &str) -> (String, String) {
    (
        format!("{SOURCE_PREFIX}{bus_name}"),
        object_path.to_string(),
    )
}

/// Returns the accessibility bus name when a service string is an AT-SPI source.
pub(super) fn atspi_bus_name(service: &str) -> Option<&str> {
    service.strip_prefix(SOURCE_PREFIX)
}

/// Finds the menu bar of the focused (or app-id/PID matching) accessibility
/// application. Returns `(accessibility bus name, menu bar object path)`.
pub(super) fn find_atspi_menubar(
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

/// Reads the full menu tree for an AT-SPI menu bar.
pub(super) fn fetch_atspi_menu(bus_name: &str, menu_path: &str) -> Option<Vec<DbusMenuItem>> {
    let conn = connect_atspi()?;
    let root = AccessibleRef {
        bus_name: bus_name.to_string(),
        object_path: menu_path.to_string(),
    };
    let mut context = BuildContext::default();
    let items = build_menu_items(&conn, &root, 0, &mut context);
    (!items.is_empty()).then_some(items)
}

/// Activates an AT-SPI menu item through `org.a11y.atspi.Action.DoAction`.
pub(super) fn activate_atspi_action(action: MenuActionRef) {
    std::thread::spawn(move || {
        let Some(conn) = connect_atspi() else {
            return;
        };
        let target = AccessibleRef {
            bus_name: action.bus_name,
            object_path: action.object_path,
        };
        let params = Variant::from((0i32,));
        let _ = call(&conn, &target, ACTION_IFACE, "DoAction", Some(&params));
    });
}

/// AT-SPI state arrays are packed 32-bit words: word 0 holds states 0..31.
fn state_has(states: &[u32], state: usize) -> bool {
    states
        .get(state / 32)
        .is_some_and(|word| word & (1u32 << (state % 32)) != 0)
}

/// Normalizes accessibility role names into the roles this module understands.
fn classify_role(role: &str) -> AccessibleRole {
    let role = role.trim().to_ascii_lowercase();
    if role == "menubar" || role.contains("menu bar") {
        AccessibleRole::MenuBar
    } else if role.contains("separator") {
        AccessibleRole::Separator
    } else if role.contains("menu item") || role.contains("menuitem") {
        AccessibleRole::MenuItem
    } else if role == "menu" || role.contains("popup menu") || role.contains("section") {
        AccessibleRole::Menu
    } else {
        AccessibleRole::Other
    }
}

fn connect_atspi() -> Option<DBusConnection> {
    let session = gio::bus_get_sync(gio::BusType::Session, gio::Cancellable::NONE).ok()?;
    let reply = session
        .call_sync(
            Some(A11Y_BUS_SERVICE),
            A11Y_BUS_PATH,
            A11Y_BUS_SERVICE,
            "GetAddress",
            None,
            None,
            gio::DBusCallFlags::NONE,
            CALL_TIMEOUT_MS,
            gio::Cancellable::NONE,
        )
        .ok()?;
    let address_value = reply.child_value(0);
    let address = address_value.str()?.to_string();
    DBusConnection::for_address_sync(
        &address,
        gio::DBusConnectionFlags::NONE,
        None,
        gio::Cancellable::NONE,
    )
    .ok()
}

fn call(
    conn: &DBusConnection,
    target: &AccessibleRef,
    interface: &str,
    method: &str,
    params: Option<&Variant>,
) -> Option<Variant> {
    conn.call_sync(
        Some(&target.bus_name),
        &target.object_path,
        interface,
        method,
        params,
        None,
        gio::DBusCallFlags::NONE,
        CALL_TIMEOUT_MS,
        gio::Cancellable::NONE,
    )
    .ok()
}

fn property_string(conn: &DBusConnection, target: &AccessibleRef, name: &str) -> Option<String> {
    let params = Variant::from((ACCESSIBLE_IFACE, name));
    let reply = call(conn, target, PROPERTIES_IFACE, "Get", Some(&params))?;
    reply.child_value(0).str().map(str::to_string)
}

fn role_name(conn: &DBusConnection, target: &AccessibleRef) -> String {
    call(conn, target, ACCESSIBLE_IFACE, "GetRoleName", None)
        .and_then(|reply| reply.child_value(0).str().map(str::to_string))
        .unwrap_or_default()
}

fn state_set(conn: &DBusConnection, target: &AccessibleRef) -> Vec<u32> {
    let Some(reply) = call(conn, target, ACCESSIBLE_IFACE, "GetState", None) else {
        return Vec::new();
    };
    let states = reply.child_value(0);
    (0..states.n_children())
        .filter_map(|index| states.child_value(index).get::<u32>())
        .collect()
}

fn accessible_children(conn: &DBusConnection, target: &AccessibleRef) -> Vec<AccessibleRef> {
    let Some(reply) = call(conn, target, ACCESSIBLE_IFACE, "GetChildren", None) else {
        return Vec::new();
    };
    let children = reply.child_value(0);
    (0..children.n_children())
        .filter_map(|index| {
            let child = children.child_value(index);
            let bus_name = variant_string(&child.child_value(0))?;
            let object_path = variant_string(&child.child_value(1))?;
            Some(AccessibleRef {
                bus_name,
                object_path,
            })
        })
        .collect()
}

fn has_action(conn: &DBusConnection, target: &AccessibleRef) -> bool {
    let Some(reply) = call(conn, target, ACCESSIBLE_IFACE, "GetInterfaces", None) else {
        return false;
    };
    let interfaces = reply.child_value(0);
    (0..interfaces.n_children()).any(|index| {
        interfaces
            .child_value(index)
            .str()
            .is_some_and(|name| name == ACTION_IFACE)
    })
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

#[cfg(test)]
mod tests {
    use super::{atspi_bus_name, classify_role, encode_atspi_source, state_has, AccessibleRole};

    #[test]
    fn source_encoding_round_trips() {
        let (service, path) = encode_atspi_source(":1.42", "/org/a11y/atspi/accessible/1");
        assert_eq!(atspi_bus_name(&service), Some(":1.42"));
        assert_eq!(path, "/org/a11y/atspi/accessible/1");
        assert_eq!(atspi_bus_name("org.example.App"), None);
    }

    #[test]
    fn states_are_packed_into_words_of_32() {
        let states = [0x0100_0001u32, 1u32 << 10, 0];
        assert!(state_has(&states, 0));
        assert!(state_has(&states, 24));
        assert!(state_has(&states, 42));
        assert!(!state_has(&states, 1));
        assert!(!state_has(&states, 64));
    }

    #[test]
    fn menu_roles_are_classified_before_generic_roles() {
        assert_eq!(classify_role("menu bar"), AccessibleRole::MenuBar);
        assert_eq!(classify_role("menuBar"), AccessibleRole::MenuBar);
        assert_eq!(classify_role("menu item"), AccessibleRole::MenuItem);
        assert_eq!(classify_role("check menu item"), AccessibleRole::MenuItem);
        assert_eq!(classify_role("separator"), AccessibleRole::Separator);
        assert_eq!(classify_role("menu"), AccessibleRole::Menu);
        assert_eq!(classify_role("window"), AccessibleRole::Other);
    }
}
