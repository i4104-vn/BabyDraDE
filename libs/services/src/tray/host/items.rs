//! Tray host items.

use super::watcher::state;
use super::*;

pub(super) fn publish_snapshot(connection: &gio::DBusConnection) {
    let keys: BTreeSet<ItemKey> =
        state()
            .lock()
            .map(|state| {
                let mut keys: BTreeSet<ItemKey> = state.registered.iter().cloned().collect();
                keys.extend(state.probed.iter().filter_map(|(service, path)| {
                    path.clone().map(|path| (service.clone(), path))
                }));
                keys
            })
            .unwrap_or_default();

    let mut items = Vec::new();
    for (service, path) in keys {
        match read_item(connection, &service, &path) {
            Some(item) => items.push(item),
            None => {
                // The item vanished (or stopped responding): drop it so a later
                // bus rescan can discover it again.
                if let Ok(mut state) = state().lock() {
                    state.registered.remove(&(service.clone(), path.clone()));
                    state.probed.remove(&service);
                }
                let _ = connection.emit_signal(
                    None,
                    WATCHER_PATH,
                    WATCHER_IFACE,
                    "StatusNotifierItemUnregistered",
                    Some(&Variant::from((service,))),
                );
            }
        }
    }

    items.sort_by(|a, b| {
        a.title
            .cmp(&b.title)
            .then_with(|| a.service.cmp(&b.service))
            .then_with(|| a.path.cmp(&b.path))
    });

    if let Ok(mut current) = snapshot().lock() {
        *current = items;
    }
}

pub(super) fn read_item(connection: &gio::DBusConnection, service: &str, path: &str) -> Option<TrayItem> {
    let reply = connection
        .call_sync(
            Some(service),
            path,
            PROPERTIES_IFACE,
            "GetAll",
            Some(&Variant::from((ITEM_IFACE,))),
            None,
            gio::DBusCallFlags::NONE,
            250,
            gio::Cancellable::NONE,
        )
        .ok()?;

    let properties = VariantDict::new(Some(&reply.child_value(0)));
    let status: String = properties.lookup("Status").ok().flatten()?;
    if status == "Passive" {
        return None;
    }

    let icon_name: String = properties
        .lookup("IconName")
        .ok()
        .flatten()
        .unwrap_or_default();
    // Some items only provide a pixmap; use a placeholder so the icon still has
    // a stable size and the tooltip remains useful.
    let icon_name = if icon_name.is_empty() {
        "image-missing".to_string()
    } else {
        icon_name
    };

    Some(TrayItem {
        service: service.to_string(),
        path: path.to_string(),
        icon_name,
        title: properties
            .lookup("Title")
            .ok()
            .flatten()
            .unwrap_or_else(|| properties.lookup("Id").ok().flatten().unwrap_or_default()),
        status,
        item_is_menu: properties
            .lookup("ItemIsMenu")
            .ok()
            .flatten()
            .unwrap_or(false),
        menu_path: properties.lookup("Menu").ok().flatten().unwrap_or_default(),
    })
}

