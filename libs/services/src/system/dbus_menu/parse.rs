//! Parsing raw D-Bus menu layout variants.

use std::collections::HashMap;

use gio::glib::Variant;

use super::DbusMenuItem;

pub(super) fn parse_layout_variant(var: &Variant) -> Option<DbusMenuItem> {
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
    let mut has_submenu = false;
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

        if let Some(display_val) = props_map.get("children-display") {
            has_submenu = extract_string(display_val)
                .is_some_and(|display| matches!(display.as_str(), "submenu" | "section"));
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
        has_submenu,
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
