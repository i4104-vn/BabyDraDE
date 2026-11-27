use super::paths::parse_registered_menu_entry;

/// Timeout for the `ListNames` call, in milliseconds.
const LIST_NAMES_TIMEOUT_MS: i32 = 400;
/// Timeout for the registrar `GetMenus` call, in milliseconds.
const REGISTRAR_TIMEOUT_MS: i32 = 250;

pub(super) fn list_bus_names(conn: &gio::DBusConnection) -> Vec<String> {
    let reply = match conn.call_sync(
        Some("org.freedesktop.DBus"),
        "/org/freedesktop/DBus",
        "org.freedesktop.DBus",
        "ListNames",
        None,
        None,
        gio::DBusCallFlags::NONE,
        LIST_NAMES_TIMEOUT_MS,
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

pub(super) fn find_registered_appmenus(conn: &gio::DBusConnection) -> Vec<(String, String)> {
    let reply = match conn.call_sync(
        Some("com.canonical.AppMenu.Registrar"),
        "/com/canonical/AppMenu/Registrar",
        "com.canonical.AppMenu.Registrar",
        "GetMenus",
        None,
        None,
        gio::DBusCallFlags::NONE,
        REGISTRAR_TIMEOUT_MS,
        gio::Cancellable::NONE,
    ) {
        Ok(reply) => reply,
        Err(_) => return Vec::new(),
    };

    let menus = reply.child_value(0);
    if !menus.is_container() {
        return Vec::new();
    }
    (0..menus.n_children())
        .filter_map(|index| parse_registered_menu_entry(&menus.child_value(index)))
        .collect()
}
