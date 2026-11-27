//! Tray host discovery.

use std::time::Instant;

use super::items::publish_snapshot;
use super::watcher::state;
use super::{DBUS_IFACE, DBUS_NAME, DBUS_PATH, ITEM_IFACE, MAX_PROBES_PER_CYCLE, RESCAN_INTERVAL};

pub(super) fn refresh(connection: &gio::DBusConnection) {
    let rescan = state()
        .lock()
        .ok()
        .and_then(|state| state.last_probe)
        .is_none_or(|last| last.elapsed() >= RESCAN_INTERVAL);

    if rescan {
        if let Ok(mut state) = state().lock() {
            state.probed.clear();
            state.last_probe = Some(Instant::now());
        }
    }

    scan_bus_names(connection);
    publish_snapshot(connection);
}

/// Probes bus names that have not been seen yet for a Status Notifier Item.
pub(super) fn scan_bus_names(connection: &gio::DBusConnection) {
    let names = list_names(connection);
    let mut budget = MAX_PROBES_PER_CYCLE;

    for name in names {
        if budget == 0 {
            break;
        }
        let known = state()
            .lock()
            .ok()
            .is_some_and(|state| state.probed.contains_key(&name));
        if known {
            continue;
        }

        budget -= 1;
        let path = find_sni_path(connection, &name);
        if let Ok(mut state) = state().lock() {
            state.probed.insert(name, path);
        }
    }
}

pub(super) fn list_names(connection: &gio::DBusConnection) -> Vec<String> {
    let reply = connection
        .call_sync(
            Some(DBUS_NAME),
            DBUS_PATH,
            DBUS_IFACE,
            "ListNames",
            None,
            None,
            gio::DBusCallFlags::NONE,
            500,
            gio::Cancellable::NONE,
        )
        .ok();
    let Some(reply) = reply else {
        return Vec::new();
    };

    let names = reply.child_value(0);
    (0..names.n_children())
        .filter_map(|index| names.child_value(index).str().map(str::to_string))
        .collect()
}

/// Finds the object path exporting `org.kde.StatusNotifierItem` on `service`.
pub(super) fn find_sni_path(connection: &gio::DBusConnection, service: &str) -> Option<String> {
    const MAX_NODES: usize = 48;
    const MAX_DEPTH: usize = 6;

    let mut pending = vec![("/".to_string(), 0usize)];
    let mut visited = 0;

    while let Some((path, depth)) = pending.pop() {
        if visited >= MAX_NODES {
            break;
        }
        visited += 1;

        let reply = connection
            .call_sync(
                Some(service),
                &path,
                "org.freedesktop.DBus.Introspectable",
                "Introspect",
                None,
                None,
                gio::DBusCallFlags::NONE,
                250,
                gio::Cancellable::NONE,
            )
            .ok()?;
        let xml = reply.child_value(0).get::<String>()?;
        let node = gio::DBusNodeInfo::for_xml(&xml).ok()?;

        if node
            .interfaces()
            .iter()
            .any(|interface| interface.name() == ITEM_IFACE)
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
