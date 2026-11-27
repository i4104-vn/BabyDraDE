//! D-Bus name/path discovery helpers.

use gio::glib::Variant;

/// Reads a string-like variant. Object paths have type `o`, so `get::<String>()`
/// cannot be used for them; `str()` accepts `s`, `o` and `g`.
pub(super) fn variant_string(var: &Variant) -> Option<String> {
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
    if !menus.is_container() {
        return Vec::new();
    }
    (0..menus.n_children())
        .filter_map(|index| parse_registered_menu_entry(&menus.child_value(index)))
        .collect()
}

/// Flattens an app id or bus name to printable lowercase characters.
pub(super) fn normalize_app_id(value: &str) -> String {
    let value = value
        .trim()
        .strip_suffix(".desktop")
        .unwrap_or(value.trim());
    value
        .to_ascii_lowercase()
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .collect()
}

/// Dot/dash/underscore separated components of an app id or bus name.
pub(super) fn app_tokens(value: &str) -> Vec<String> {
    let value = value.trim();
    let value = value.strip_suffix(".desktop").unwrap_or(value);
    value
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|token| token.len() >= 2)
        .map(str::to_ascii_lowercase)
        .collect()
}

/// Matches a bus name against an app id without hardcoding any application.
///
/// Handles desktop ids (`org.gnome.Nautilus.desktop`), short executable names
/// (`code` vs `com.visualstudio.code`), and per-instance suffixes
/// (`org.kde.kate_4578`) used by KDE and Electron helpers.
pub(super) fn service_matches_app_id(service: &str, app_id: &str) -> bool {
    let service_normalized = normalize_app_id(service);
    let app_normalized = normalize_app_id(app_id);
    if service_normalized.is_empty() || app_normalized.is_empty() {
        return false;
    }
    if service_normalized == app_normalized {
        return true;
    }
    if app_normalized.len() >= 3
        && (service_normalized.starts_with(&app_normalized)
            || service_normalized.ends_with(&app_normalized))
    {
        return true;
    }

    let app_id_tokens = app_tokens(app_id);
    let service_tokens = app_tokens(service);
    // The last token is the application's own name; namespace tokens such as
    // `org` or `gnome` are shared by unrelated apps and must not count.
    let Some(distinctive) = app_id_tokens.iter().rev().find(|token| token.len() >= 3) else {
        return false;
    };
    service_tokens.iter().any(|candidate| {
        candidate.starts_with(distinctive)
            || (distinctive.starts_with(candidate) && candidate.len() >= 4)
    })
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

    use super::{
        app_tokens, normalize_app_id, parse_registered_menu_entry, service_matches_app_id,
    };

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

    #[test]
    fn app_id_normalization_keeps_useful_tokens() {
        assert_eq!(
            normalize_app_id("org.gnome.Nautilus.desktop"),
            "orggnomenautilus"
        );
        assert_eq!(
            app_tokens("firefox_firefox.desktop"),
            vec!["firefox", "firefox"]
        );
    }

    #[test]
    fn service_matching_accepts_desktop_and_helper_name_variants() {
        assert!(service_matches_app_id(
            "org.gnome.Nautilus",
            "org.gnome.Nautilus.desktop"
        ));
        assert!(service_matches_app_id("com.visualstudio.code", "code"));
        assert!(service_matches_app_id("org.kde.kate_4578", "org.kde.kate"));
        assert!(service_matches_app_id("firefox", "firefox_firefox.desktop"));
        assert!(!service_matches_app_id(
            "org.gnome.Terminal",
            "org.gnome.Nautilus"
        ));
    }
}
