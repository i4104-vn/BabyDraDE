//! D-Bus name/path discovery helpers.

use gio::glib::Variant;


pub(super) fn find_registered_appmenus(conn: &gio::DBusConnection) -> Vec<(String, String)> {
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
            matches.push((service, path));
        }
    }

    matches
}

pub(super) fn service_matches_app_id(service: &str, app_id: &str) -> bool {
    let service = service.trim().to_ascii_lowercase();
    let app_id = app_id
        .trim()
        .strip_suffix(".desktop")
        .unwrap_or(app_id.trim())
        .to_ascii_lowercase();
    if service.is_empty() || app_id.is_empty() {
        return false;
    }
    if service == app_id || service.ends_with(&format!(".{app_id}")) {
        return true;
    }

    let normalized_service: String = service
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .collect();
    let normalized_app: String = app_id.chars().filter(char::is_ascii_alphanumeric).collect();
    normalized_app.len() >= 3
        && (normalized_service == normalized_app
            || normalized_service.starts_with(&normalized_app)
            || normalized_service.ends_with(&normalized_app))
}

pub(super) fn get_connection_process_id(conn: &gio::DBusConnection, name: &str) -> Option<u32> {
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

pub(super) fn find_dbusmenu_path(conn: &gio::DBusConnection, service: &str) -> Option<String> {
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

    None
}



#[cfg(test)]
mod tests {
    use super::service_matches_app_id;

    #[test]
    fn service_matching_accepts_desktop_and_helper_name_variants() {
        assert!(service_matches_app_id(
            "org.gnome.Nautilus",
            "org.gnome.Nautilus.desktop"
        ));
        assert!(service_matches_app_id("com.visualstudio.code", "code"));
        assert!(service_matches_app_id("org.kde.kate_4578", "org.kde.kate"));
        assert!(!service_matches_app_id(
            "org.gnome.Terminal",
            "org.gnome.Nautilus"
        ));
    }
}
