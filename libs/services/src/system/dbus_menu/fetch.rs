//! Fetching and activating D-Bus menu items.

use std::borrow::Cow;

use gio::glib::Variant;

use super::atspi::{activate_atspi_action, atspi_bus_name, fetch_atspi_menu};
use super::parse::parse_layout_variant;
use super::{DbusMenuItem, MenuActionRef};

/// How many lazy submenus are prepared before a fetch; bounds work on large menus.
const SUBMENU_PREPARE_LIMIT: usize = 48;
/// Timeout for the cheap layout probe used by discovery, in milliseconds.
const PROBE_TIMEOUT_MS: i32 = 180;
/// Timeout for a full layout fetch, in milliseconds.
const FETCH_TIMEOUT_MS: i32 = 400;
/// Timeout for an `AboutToShow` call, in milliseconds.
const ABOUT_TO_SHOW_TIMEOUT_MS: i32 = 300;
/// Timeout for an `Event` activation call, in milliseconds.
const ACTIVATE_TIMEOUT_MS: i32 = 500;

/// Fetches the hierarchical DBus menu from an application exporting `com.canonical.dbusmenu`
pub fn fetch_dbus_menu(service: &str, path: &str) -> Option<Vec<DbusMenuItem>> {
    let conn = gio::bus_get_sync(gio::BusType::Session, gio::Cancellable::NONE).ok()?;
    fetch_dbus_menu_with_connection(&conn, service, path)
}

/// Fetches a menu after asking the exporter to populate lazy submenus.
///
/// Many GTK, Qt, Electron, and Java app-menu exporters only provide the children of
/// File/Edit/etc. after receiving the `AboutToShow` protocol event. Without this
/// preparation the menu bar appears but its menus look empty or act like inert buttons.
pub fn fetch_dbus_menu_with_prepared_submenus(
    service: &str,
    path: &str,
) -> Option<Vec<DbusMenuItem>> {
    let conn = gio::bus_get_sync(gio::BusType::Session, gio::Cancellable::NONE).ok()?;
    // Some exporters lazily create the top-level categories as well.
    let _ = about_to_show_with_connection(&conn, service, path, 0);
    let initial_items = fetch_dbus_menu_with_connection(&conn, service, path)?;

    let mut submenu_ids = Vec::new();
    collect_submenu_ids(&initial_items, &mut submenu_ids);
    for item_id in submenu_ids.into_iter().take(SUBMENU_PREPARE_LIMIT) {
        // A few exporters do not implement AboutToShow. Their static layout remains usable.
        let _ = about_to_show_with_connection(&conn, service, path, item_id);
    }

    fetch_dbus_menu_with_connection(&conn, service, path).or(Some(initial_items))
}

/// Cheap, non-mutating layout read used while probing discovery candidates.
/// It deliberately skips `AboutToShow` so one bad candidate cannot delay the
/// whole discovery pass.
pub(super) fn probe_dbus_menu(service: &str, path: &str) -> Option<Vec<DbusMenuItem>> {
    let conn = gio::bus_get_sync(gio::BusType::Session, gio::Cancellable::NONE).ok()?;
    let reply = get_layout(&conn, service, path, PROBE_TIMEOUT_MS)?;
    parse_layout_reply(&reply)
}

fn fetch_dbus_menu_with_connection(
    conn: &gio::DBusConnection,
    service: &str,
    path: &str,
) -> Option<Vec<DbusMenuItem>> {
    let reply = get_layout(conn, service, path, FETCH_TIMEOUT_MS)?;
    parse_layout_reply(&reply)
}

fn get_layout(
    conn: &gio::DBusConnection,
    service: &str,
    path: &str,
    timeout_ms: i32,
) -> Option<Variant> {
    // Read the complete tree so overflow menus and nested submenus have all entries available.
    let params = Variant::from((
        0i32,
        -1i32,
        &[
            "label",
            "enabled",
            "visible",
            "type",
            "toggle-type",
            "toggle-state",
            "children-display",
        ][..],
    ));

    conn.call_sync(
        Some(service),
        path,
        "com.canonical.dbusmenu",
        "GetLayout",
        Some(&params),
        None,
        gio::DBusCallFlags::NONE,
        timeout_ms,
        gio::Cancellable::NONE,
    )
    .ok()
}

fn parse_layout_reply(reply: &Variant) -> Option<Vec<DbusMenuItem>> {
    // Signature is (u(ia{sv}av)). A few exporters omit the revision argument.
    let root_layout: Cow<'_, Variant> = match reply.n_children() {
        0 => return None,
        1 => Cow::Owned(reply.child_value(0)),
        2 => Cow::Owned(reply.child_value(1)),
        _ if reply.child_value(0).get::<i32>().is_some() => Cow::Borrowed(reply),
        _ => Cow::Owned(reply.child_value(1)),
    };
    let root_item = parse_layout_variant(&root_layout)?;
    Some(root_item.children)
}

/// Fetches the menu for a discovery handle, dispatching to D-Bus or AT-SPI.
pub fn fetch_app_menu(service: &str, path: &str) -> Option<Vec<DbusMenuItem>> {
    if let Some(bus_name) = atspi_bus_name(service) {
        fetch_atspi_menu(bus_name, path)
    } else {
        fetch_dbus_menu_with_prepared_submenus(service, path)
    }
}

/// Activates a menu item from either menu source.
pub fn activate_app_menu_item(
    service: &str,
    path: &str,
    item_id: i32,
    action: Option<MenuActionRef>,
) {
    if atspi_bus_name(service).is_some() {
        if let Some(action) = action {
            activate_atspi_action(action);
        }
    } else {
        activate_dbus_menu_item(service, path, item_id);
    }
}

fn about_to_show_with_connection(
    conn: &gio::DBusConnection,
    service: &str,
    path: &str,
    item_id: i32,
) -> bool {
    let params = Variant::from((item_id,));
    conn.call_sync(
        Some(service),
        path,
        "com.canonical.dbusmenu",
        "AboutToShow",
        Some(&params),
        None,
        gio::DBusCallFlags::NONE,
        ABOUT_TO_SHOW_TIMEOUT_MS,
        gio::Cancellable::NONE,
    )
    .is_ok()
}

fn collect_submenu_ids(items: &[DbusMenuItem], ids: &mut Vec<i32>) {
    for item in items {
        // Older exporters omit `children-display` until a menu is first opened.
        // AboutToShow is harmless for leaf entries and lets those menus reveal
        // their children before the topbar decides whether to render a submenu.
        if item.visible && !item.is_separator {
            ids.push(item.id);
        }
        collect_submenu_ids(&item.children, ids);
    }
}

/// Activates a menu item on the application's D-Bus service
pub fn activate_dbus_menu_item(service: &str, path: &str, item_id: i32) {
    let service = service.to_string();
    let path = path.to_string();

    std::thread::spawn(move || {
        if let Ok(conn) = gio::bus_get_sync(gio::BusType::Session, gio::Cancellable::NONE) {
            let empty_val = Variant::from("");
            let params = Variant::from((item_id, "clicked", empty_val, 0u32));

            let _ = conn.call_sync(
                Some(&service),
                &path,
                "com.canonical.dbusmenu",
                "Event",
                Some(&params),
                None,
                gio::DBusCallFlags::NONE,
                ACTIVATE_TIMEOUT_MS,
                gio::Cancellable::NONE,
            );
        }
    });
}
