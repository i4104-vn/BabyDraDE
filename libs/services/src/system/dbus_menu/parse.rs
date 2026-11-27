//! Parsing raw D-Bus menu layout variants.

use std::borrow::Cow;
use std::collections::HashMap;

use gio::glib::Variant;

use super::DbusMenuItem;

/// Removes toolkit mnemonics (`_` in GTK, `&` in Qt) and trims whitespace.
pub(super) fn normalize_menu_label(raw: &str) -> String {
    raw.replace(&['_', '&'][..], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Reads a boolean-ish property. Exporters variously send `b`, `i`, `u` or
/// string values for `enabled`/`visible`, so accept all of them.
pub(super) fn property_bool(props: &HashMap<String, Variant>, keys: &[&str]) -> Option<bool> {
    for key in keys {
        let Some(value) = props.get(*key) else {
            continue;
        };
        if let Some(value) = value.get::<bool>() {
            return Some(value);
        }
        if let Some(value) = value.get::<i32>() {
            return Some(value != 0);
        }
        if let Some(value) = value.get::<u32>() {
            return Some(value != 0);
        }
        if let Some(value) = extract_string(value) {
            return Some(matches!(
                value.to_ascii_lowercase().as_str(),
                "true" | "yes" | "1"
            ));
        }
    }
    None
}

/// Separators are advertised as `type == "separator"` by most exporters, but
/// a few use `"sep"` or omit `type` and only set an empty label.
pub(super) fn is_separator_type(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "separator" | "sep"
    )
}

pub(super) fn is_submenu_display(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "submenu" | "section" | "menu"
    )
}

/// Maximum nesting accepted from an exporter's layout variant. Prevents a
/// deeply nested (or malicious) D-Bus message from exhausting the stack.
const MAX_LAYOUT_DEPTH: usize = 16;

/// Checks the whole subtree before parsing so one over-deep child rejects the
/// entire layout instead of being silently dropped by its parent.
fn layout_too_deep(var: &Variant, depth: usize) -> bool {
    if depth > MAX_LAYOUT_DEPTH {
        return true;
    }
    let actual: Cow<'_, Variant> = if var.is_container() && var.type_().as_str() == "v" {
        Cow::Owned(var.child_value(0))
    } else {
        Cow::Borrowed(var)
    };
    if actual.n_children() < 3 {
        return false;
    }
    let children = actual.child_value(2);
    (0..children.n_children()).any(|i| layout_too_deep(&children.child_value(i), depth + 1))
}

pub(super) fn parse_layout_variant(var: &Variant) -> Option<DbusMenuItem> {
    if layout_too_deep(var, 0) {
        return None;
    }
    parse_layout_variant_inner(var, 0)
}

fn parse_layout_variant_inner(var: &Variant, depth: usize) -> Option<DbusMenuItem> {
    if depth > MAX_LAYOUT_DEPTH {
        return None;
    }

    let actual_var: Cow<'_, Variant> = if var.is_container() && var.type_().as_str() == "v" {
        Cow::Owned(var.child_value(0))
    } else {
        Cow::Borrowed(var)
    };

    if actual_var.n_children() < 3 {
        return None;
    }

    let id = actual_var
        .child_value(0)
        .get::<i32>()
        .or_else(|| actual_var.child_value(0).get::<u32>().map(|id| id as i32))?;
    let props_var = actual_var.child_value(1);
    let children_var = actual_var.child_value(2);

    let mut label = String::new();
    let mut enabled = true;
    let mut visible = true;
    let mut is_separator = false;
    let mut has_submenu = false;
    let mut toggle_type = None;
    let mut toggle_state = None;

    // Parse dictionary properties a{sv}. Some exporters capitalize keys or use
    // `title`/`name` instead of `label`, so probe the common spellings.
    if let Some(props_map) = props_var.get::<HashMap<String, Variant>>() {
        for key in ["label", "Label", "title", "name", "text"] {
            if let Some(value) = props_map.get(key) {
                if let Some(text) = extract_string(value) {
                    label = normalize_menu_label(&text);
                    break;
                }
            }
        }

        if let Some(value) = property_bool(&props_map, &["enabled", "sensitive"]) {
            enabled = value;
        }
        if let Some(value) = property_bool(&props_map, &["visible", "showing"]) {
            visible = value;
        }

        if let Some(value) = props_map.get("type").and_then(extract_string) {
            is_separator = is_separator_type(&value);
        }

        if let Some(value) = props_map
            .get("children-display")
            .or_else(|| props_map.get("display"))
            .and_then(extract_string)
        {
            has_submenu = is_submenu_display(&value);
        }

        if let Some(value) = props_map.get("toggle-type") {
            toggle_type = extract_string(value);
        }

        if let Some(value) = props_map.get("toggle-state") {
            toggle_state = value
                .get::<i32>()
                .or_else(|| value.get::<u32>().map(|state| state as i32));
        }
    }

    // Parse children array av
    let mut children = Vec::new();
    for i in 0..children_var.n_children() {
        let child_val = children_var.child_value(i);
        if let Some(parsed_child) = parse_layout_variant_inner(&child_val, depth + 1) {
            if parsed_child.visible {
                children.push(parsed_child);
            }
        }
    }

    if label.is_empty() && !is_separator && children.is_empty() {
        has_submenu = false;
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
        action: None,
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use gio::glib::Variant;

    use super::{
        is_separator_type, is_submenu_display, normalize_menu_label, parse_layout_variant,
        property_bool, MAX_LAYOUT_DEPTH,
    };

    #[test]
    fn labels_are_stripped_of_toolkit_mnemonics() {
        assert_eq!(normalize_menu_label("_File"), "File");
        assert_eq!(normalize_menu_label("Save & Exit"), "Save Exit");
        assert_eq!(normalize_menu_label("  New _Window  "), "New Window");
    }

    #[test]
    fn separator_and_submenu_properties_accept_variants() {
        assert!(is_separator_type("separator"));
        assert!(is_separator_type(" Sep "));
        assert!(!is_separator_type("menu"));
        assert!(is_submenu_display("submenu"));
        assert!(is_submenu_display("section"));
        assert!(is_submenu_display("MENU"));
        assert!(!is_submenu_display(""));
    }

    #[test]
    fn boolean_properties_accept_bool_int_and_string_values() {
        let mut props = HashMap::new();
        props.insert("enabled".to_string(), Variant::from(false));
        props.insert("visible".to_string(), Variant::from(1i32));
        props.insert("sensitive".to_string(), Variant::from("yes"));
        assert_eq!(property_bool(&props, &["enabled"]), Some(false));
        assert_eq!(property_bool(&props, &["visible"]), Some(true));
        assert_eq!(property_bool(&props, &["sensitive"]), Some(true));
        assert_eq!(property_bool(&props, &["missing"]), None);
    }

    fn nested_layout(depth: usize) -> Variant {
        let mut children: Vec<Variant> = Vec::new();
        for _ in 0..depth {
            let item = Variant::from((1i32, HashMap::<String, Variant>::new(), children));
            children = vec![item];
        }
        Variant::from((1i32, HashMap::<String, Variant>::new(), children))
    }

    #[test]
    fn deeply_nested_layout_is_rejected() {
        assert!(parse_layout_variant(&nested_layout(2)).is_some());
        assert!(parse_layout_variant(&nested_layout(MAX_LAYOUT_DEPTH + 5)).is_none());
    }
}
