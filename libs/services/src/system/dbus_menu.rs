//! DBus Menu Protocol Implementation (com.canonical.dbusmenu)

use gio::glib::Variant;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct DbusMenuItem {
    pub id: i32,
    pub label: String,
    pub enabled: bool,
    pub visible: bool,
    pub is_separator: bool,
    pub toggle_type: Option<String>,
    pub toggle_state: Option<i32>,
    pub children: Vec<DbusMenuItem>,
}

/// Fetches the hierarchical DBus menu from an application exporting `com.canonical.dbusmenu`
pub fn fetch_dbus_menu(service: &str, path: &str) -> Option<Vec<DbusMenuItem>> {
    let conn = gio::bus_get_sync(gio::BusType::Session, gio::Cancellable::NONE).ok()?;

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

    let reply = conn
        .call_sync(
            Some(service),
            path,
            "com.canonical.dbusmenu",
            "GetLayout",
            Some(&params),
            None,
            gio::DBusCallFlags::NONE,
            400,
            gio::Cancellable::NONE,
        )
        .ok()?;

    // Signature is (u(ia{sv}av))
    if reply.n_children() < 2 {
        return None;
    }

    let root_layout = reply.child_value(1);
    let root_item = parse_layout_variant(&root_layout)?;
    Some(root_item.children)
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
                500,
                gio::Cancellable::NONE,
            );
        }
    });
}

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
            if name_lower == app_id_clean {
                exact_name_candidates.push(name.to_string());
            } else if name_lower.ends_with(&format!(".{app_id_clean}")) {
                suffix_candidates.push(name.to_string());
            }
        }
    }

    if let Some(process_id) = process_id {
        process_candidates.extend(find_registered_appmenus(&conn, process_id));

        // The focused Wayland client's PID is the strongest available association
        // between an xdg_toplevel and a D-Bus connection.
        for unique_name in unique_names {
            if get_connection_process_id(&conn, &unique_name) == Some(process_id) {
                process_candidates.push((unique_name, String::new()));
            }
        }
    }

    let mut candidates = process_candidates;
    candidates.extend(
        exact_name_candidates
            .into_iter()
            .map(|service| (service, String::new())),
    );
    if process_id.is_none() {
        candidates.extend(
            suffix_candidates
                .into_iter()
                .map(|service| (service, String::new())),
        );
    }
    let mut seen = std::collections::HashSet::new();

    for (service, registered_path) in candidates {
        if !seen.insert(service.clone()) && registered_path.is_empty() {
            continue;
        }

        if !registered_path.is_empty()
            && fetch_dbus_menu(&service, &registered_path).is_some_and(|items| !items.is_empty())
        {
            return Some((service, registered_path));
        }

        for path in CANDIDATE_PATHS {
            if fetch_dbus_menu(&service, path).is_some_and(|items| !items.is_empty()) {
                return Some((service, path.to_string()));
            }
        }

        if let Some(path) = find_dbusmenu_path(&conn, &service) {
            if fetch_dbus_menu(&service, &path).is_some_and(|items| !items.is_empty()) {
                return Some((service, path));
            }
        }
    }

    None
}

fn find_registered_appmenus(conn: &gio::DBusConnection, process_id: u32) -> Vec<(String, String)> {
    let reply = match conn.call_sync(
        Some("com.canonical.AppMenu.Registrar"),
        "/com/canonical/AppMenu/Registrar",
        "com.canonical.AppMenu.Registrar",
        "GetMenus",
        None,
        None,
        gio::DBusCallFlags::NONE,
        250,
        gio::Cancellable::NONE,
    ) {
        Ok(reply) => reply,
        Err(_) => return Vec::new(),
    };

    let menus = reply.child_value(0);
    let mut matches = Vec::new();

    for i in 0..menus.n_children() {
        let menu = menus.child_value(i);
        if menu.n_children() < 3 {
            continue;
        }

        let service = menu.child_value(1).get::<String>();
        let path = menu.child_value(2).get::<String>();
        if let (Some(service), Some(path)) = (service, path) {
            if get_connection_process_id(conn, &service) == Some(process_id) {
                matches.push((service, path));
            }
        }
    }

    matches
}

fn get_connection_process_id(conn: &gio::DBusConnection, name: &str) -> Option<u32> {
    let params = Variant::from((name,));
    conn.call_sync(
        Some("org.freedesktop.DBus"),
        "/org/freedesktop/DBus",
        "org.freedesktop.DBus",
        "GetConnectionUnixProcessID",
        Some(&params),
        None,
        gio::DBusCallFlags::NONE,
        100,
        gio::Cancellable::NONE,
    )
    .ok()?
    .child_value(0)
    .get::<u32>()
}

fn find_dbusmenu_path(conn: &gio::DBusConnection, service: &str) -> Option<String> {
    const MAX_NODES: usize = 64;
    const MAX_DEPTH: usize = 8;

    let mut pending = vec![("/".to_string(), 0usize)];
    let mut visited = 0;

    while let Some((path, depth)) = pending.pop() {
        if visited >= MAX_NODES {
            break;
        }
        visited += 1;

        let reply = match conn.call_sync(
            Some(service),
            &path,
            "org.freedesktop.DBus.Introspectable",
            "Introspect",
            None,
            None,
            gio::DBusCallFlags::NONE,
            200,
            gio::Cancellable::NONE,
        ) {
            Ok(reply) => reply,
            Err(_) => continue,
        };
        let Some(xml) = reply.child_value(0).get::<String>() else {
            continue;
        };
        let Ok(node) = gio::DBusNodeInfo::for_xml(&xml) else {
            continue;
        };

        if node
            .interfaces()
            .iter()
            .any(|interface| interface.name() == "com.canonical.dbusmenu")
        {
            return Some(path);
        }

        if depth < MAX_DEPTH {
            for child in node.nodes() {
                if let Some(child_path) = child.path() {
                    let child_path = if child_path.starts_with('/') {
                        child_path.to_string()
                    } else if path == "/" {
                        format!("/{child_path}")
                    } else {
                        format!("{path}/{child_path}")
                    };
                    pending.push((child_path, depth + 1));
                }
            }
        }
    }

    None
}

fn parse_layout_variant(var: &Variant) -> Option<DbusMenuItem> {
    let actual_var = if var.is_container() && var.type_().as_str() == "v" {
        var.child_value(0)
    } else {
        var.clone()
    };

    if actual_var.n_children() < 3 {
        return None;
    }

    let id = actual_var.child_value(0).get::<i32>()?;
    let props_var = actual_var.child_value(1);
    let children_var = actual_var.child_value(2);

    let mut label = String::new();
    let mut enabled = true;
    let mut visible = true;
    let mut is_separator = false;
    let mut toggle_type = None;
    let mut toggle_state = None;

    // Parse dictionary properties a{sv}
    if let Some(props_map) = props_var.get::<HashMap<String, Variant>>() {
        if let Some(lbl_val) = props_map.get("label") {
            if let Some(s) = extract_string(lbl_val) {
                label = s.replace('_', "");
            }
        }

        if let Some(en_val) = props_map.get("enabled") {
            if let Some(b) = en_val.get::<bool>() {
                enabled = b;
            }
        }

        if let Some(vis_val) = props_map.get("visible") {
            if let Some(b) = vis_val.get::<bool>() {
                visible = b;
            }
        }

        if let Some(type_val) = props_map.get("type") {
            if let Some(s) = extract_string(type_val) {
                is_separator = s == "separator";
            }
        }

        if let Some(t_val) = props_map.get("toggle-type") {
            toggle_type = extract_string(t_val);
        }

        if let Some(s_val) = props_map.get("toggle-state") {
            toggle_state = s_val.get::<i32>();
        }
    }

    // Parse children array av
    let mut children = Vec::new();
    let n_children = children_var.n_children();
    for i in 0..n_children {
        let child_val = children_var.child_value(i);
        if let Some(parsed_child) = parse_layout_variant(&child_val) {
            if parsed_child.visible {
                children.push(parsed_child);
            }
        }
    }

    Some(DbusMenuItem {
        id,
        label,
        enabled,
        visible,
        is_separator,
        toggle_type,
        toggle_state,
        children,
    })
}

fn extract_string(var: &Variant) -> Option<String> {
    if let Some(s) = var.get::<String>() {
        return Some(s);
    }
    if let Some(s) = var.str() {
        return Some(s.to_string());
    }
    if var.is_container() && var.type_().as_str() == "v" {
        return extract_string(&var.child_value(0));
    }
    None
}
