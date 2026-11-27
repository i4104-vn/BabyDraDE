//! D-Bus name/path discovery helpers.

use gio::glib::Variant;

/// Timeout for an `Introspect` call, in milliseconds.
const INTROSPECT_TIMEOUT_MS: i32 = 200;

/// Reads a string-like variant. Object paths have type `o`, so `get::<String>()`
/// cannot be used for them; `str()` accepts `s`, `o` and `g`.
pub(in crate::system::dbus_menu) fn variant_string(var: &Variant) -> Option<String> {
    var.str()
        .map(str::to_string)
        .or_else(|| var.get::<String>())
}

/// `GetMenus` returns `a(so)`. Older Qt/KDE registrars returned `a(sso)` or
/// `a(uso)` (window id first), so tolerate both layouts.
pub(super) fn parse_registered_menu_entry(entry: &Variant) -> Option<(String, String)> {
    if !entry.is_container() {
        return None;
    }
    let children = entry.n_children();
    if children < 2 {
        return None;
    }
    let (service_index, path_index) = if children >= 3 { (1, 2) } else { (0, 1) };
    let service = variant_string(&entry.child_value(service_index))?;
    let path = variant_string(&entry.child_value(path_index))?;
    if service.is_empty() || !path.starts_with('/') {
        return None;
    }
    Some((service, path))
}

/// Returns every object path exporting `com.canonical.dbusmenu`, shallowest first.
pub(super) fn find_dbusmenu_paths(
    conn: &gio::DBusConnection,
    service: &str,
) -> Vec<(String, usize)> {
    const MAX_NODES: usize = 96;
    const MAX_DEPTH: usize = 8;
    const MAX_RESULTS: usize = 16;

    let mut pending = vec![("/".to_string(), 0usize)];
    let mut visited = 0;
    let mut matches = Vec::new();

    while let Some((path, depth)) = pending.pop() {
        if visited >= MAX_NODES || matches.len() >= MAX_RESULTS {
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
            INTROSPECT_TIMEOUT_MS,
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
            matches.push((path, depth));
            continue;
        }

        if depth < MAX_DEPTH {
            for child in node.nodes() {
                if let Some(child_path) = child.path() {
                    let child_path = match true {
                        _ if child_path.starts_with('/') => child_path.to_string(),
                        _ if path == "/" => format!("/{child_path}"),
                        _ => format!("{path}/{child_path}"),
                    };
                    pending.push((child_path, depth + 1));
                }
            }
        }
    }

    matches.sort_by_key(|(_, depth)| *depth);
    matches
}

#[cfg(test)]
mod tests {
    use gio::glib::Variant;

    use super::parse_registered_menu_entry;

    #[test]
    fn registrar_entries_accept_both_a_so_and_a_sso_shapes() {
        let pair = Variant::from((
            "org.example.App".to_string(),
            "/com/canonical/menu".to_string(),
        ));
        assert_eq!(
            parse_registered_menu_entry(&pair),
            Some((
                "org.example.App".to_string(),
                "/com/canonical/menu".to_string()
            ))
        );

        let triple = Variant::from((42u32, "org.example.App".to_string(), "/MenuBar".to_string()));
        assert_eq!(
            parse_registered_menu_entry(&triple),
            Some(("org.example.App".to_string(), "/MenuBar".to_string()))
        );

        assert_eq!(parse_registered_menu_entry(&Variant::from(1u32)), None);
    }
}
