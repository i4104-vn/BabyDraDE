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

    // Call GetLayout(0, 3, ["label", "enabled", "visible", "type", "toggle-type", "toggle-state"])
    let params = Variant::from((
        0i32,
        3i32,
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
            800,
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
