//! Status Notifier Host internals: watcher D-Bus object, item discovery and
//! property polling. Everything here runs on the tray host thread only.

use super::{snapshot, HostState, ItemKey, TrayItem, HOST_STATE};
use gio::glib::{Variant, VariantDict};
use std::collections::BTreeSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub(super) const ITEM_IFACE: &str = "org.kde.StatusNotifierItem";

const WATCHER_NAME: &str = "org.kde.StatusNotifierWatcher";
const WATCHER_PATH: &str = "/StatusNotifierWatcher";
const WATCHER_IFACE: &str = "org.kde.StatusNotifierWatcher";
const PROPERTIES_IFACE: &str = "org.freedesktop.DBus.Properties";
const DBUS_NAME: &str = "org.freedesktop.DBus";
const DBUS_PATH: &str = "/org/freedesktop/DBus";
const DBUS_IFACE: &str = "org.freedesktop.DBus";

const REFRESH_INTERVAL: Duration = Duration::from_millis(500);
const RESCAN_INTERVAL: Duration = Duration::from_secs(10);
const MAX_PROBES_PER_CYCLE: usize = 48;

const WATCHER_XML: &str = r#"
<node>
  <interface name="org.kde.StatusNotifierWatcher">
    <method name="RegisterStatusNotifierItem">
      <arg type="s" name="identifier" direction="in"/>
    </method>
    <method name="RegisterStatusNotifierHost">
      <arg type="s" name="service" direction="in"/>
    </method>
    <signal name="StatusNotifierItemRegistered">
      <arg type="s" name="service"/>
    </signal>
    <signal name="StatusNotifierItemUnregistered">
      <arg type="s" name="service"/>
    </signal>
    <signal name="StatusNotifierHostRegistered"/>
    <property name="RegisteredStatusNotifierItems" type="as" access="read"/>
    <property name="IsStatusNotifierHostRegistered" type="b" access="read"/>
    <property name="ProtocolVersion" type="i" access="read"/>
  </interface>
</node>
"#;

fn state() -> &'static Mutex<HostState> {
    HOST_STATE.get_or_init(|| Mutex::new(HostState::default()))
}

/// Entry point of the dedicated tray host thread.
pub(super) fn run() {
    let context = glib::MainContext::new();
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);

    // The connection and every D-Bus callback must be tied to the same main
    // context. `with_thread_default` acquires the context for the duration of
    // the closure, so the main loop can only be started afterwards.
    let setup = context.with_thread_default(|| {
        let Some(connection) =
            gio::bus_get_sync(gio::BusType::Session, gio::Cancellable::NONE).ok()
        else {
            tracing::debug!("no session bus available; tray icons are disabled");
            let _ = sender.send(None);
            return;
        };

        let owns_watcher = request_watcher_name(&connection);
        if let Err(err) = register_watcher(&connection) {
            tracing::warn!("failed to export status notifier watcher: {err}");
        }
        if !owns_watcher {
            // Another panel already owns the watcher name. Items registered
            // with it are still picked up by the bus scan below.
            tracing::debug!("status notifier watcher name is owned by another host");
        }

        // `timeout_add_local_full` always targets the *global* default context,
        // which belongs to GTK. Attach the source to our own context explicitly.
        let timer_connection = connection.clone();
        let refresh_source = glib::timeout_source_new(
            REFRESH_INTERVAL,
            Some("babydra-tray-refresh"),
            glib::Priority::DEFAULT,
            move || {
                refresh(&timer_connection);
                glib::ControlFlow::Continue
            },
        );
        refresh_source.attach(Some(&context));

        let _ = sender.send(Some(connection));
    });
    if let Err(err) = setup {
        tracing::warn!("failed to configure tray host main context: {err}");
        return;
    }

    let Ok(Some(_connection)) = receiver.recv() else {
        return;
    };
    // Keep the connection alive: GIO closes it when the last reference drops.
    let main_loop = glib::MainLoop::new(Some(&context), false);
    main_loop.run();
}

/// Requests the well-known watcher name. Returns true when we are (or already
/// were) the primary owner.
fn request_watcher_name(connection: &gio::DBusConnection) -> bool {
    // 4 = DBUS_REQUEST_NAME_FLAG_DO_NOT_QUEUE: never steal a name from an
    // already running panel (e.g. after a restart of the topbar).
    let parameters = Variant::from((WATCHER_NAME, 4u32));
    let reply = connection.call_sync(
        Some(DBUS_NAME),
        DBUS_PATH,
        DBUS_IFACE,
        "RequestName",
        Some(&parameters),
        None,
        gio::DBusCallFlags::NONE,
        1000,
        gio::Cancellable::NONE,
    );

    match reply
        .ok()
        .and_then(|reply| reply.child_value(0).get::<u32>())
    {
        Some(1 | 3) => true,
        Some(code) => {
            tracing::debug!("could not own {WATCHER_NAME} (reply {code})");
            false
        }
        None => {
            tracing::debug!("could not request {WATCHER_NAME}");
            false
        }
    }
}

fn register_watcher(connection: &gio::DBusConnection) -> Result<(), glib::Error> {
    let node = gio::DBusNodeInfo::for_xml(WATCHER_XML)?;
    let interface = node
        .interfaces()
        .first()
        .cloned()
        .expect("watcher XML defines one interface");

    let host_registered = Arc::new(AtomicBool::new(true));
    let registered_for_method = host_registered.clone();
    let registered_for_property = host_registered.clone();

    let watcher_connection = connection.clone();
    connection
        .register_object(WATCHER_PATH, &interface)
        .method_call(
            move |_conn, sender, _path, _interface, method, parameters, invocation| match method {
                "RegisterStatusNotifierItem" => {
                    let identifier_variant = parameters.child_value(0);
                    let Some(identifier) = identifier_variant.str() else {
                        invocation.return_dbus_error(
                            "org.kde.StatusNotifierWatcher.Error.InvalidArgs",
                            "missing item identifier",
                        );
                        return;
                    };

                    let key = normalize_identifier(sender, identifier);
                    if let Ok(mut state) = state().lock() {
                        state.registered.insert(key.clone());
                        state.probed.insert(key.0.clone(), Some(key.1.clone()));
                    }
                    tracing::debug!("tray item registered: {}:{}", key.0, key.1);

                    let _ = watcher_connection.emit_signal(
                        None,
                        WATCHER_PATH,
                        WATCHER_IFACE,
                        "StatusNotifierItemRegistered",
                        Some(&Variant::from((key.0,))),
                    );
                    invocation.return_value(None);
                }
                "RegisterStatusNotifierHost" => {
                    registered_for_method.store(true, Ordering::Relaxed);
                    let _ = watcher_connection.emit_signal(
                        None,
                        WATCHER_PATH,
                        WATCHER_IFACE,
                        "StatusNotifierHostRegistered",
                        None,
                    );
                    invocation.return_value(None);
                }
                _ => invocation.return_dbus_error(
                    "org.freedesktop.DBus.Error.UnknownMethod",
                    "unknown watcher method",
                ),
            },
        )
        .property(
            move |_conn, _sender, _path, _interface, property| match property {
                "RegisteredStatusNotifierItems" => {
                    let items: Vec<String> = state()
                        .lock()
                        .map(|state| {
                            state
                                .registered
                                .iter()
                                .map(|(_, path)| path.clone())
                                .collect()
                        })
                        .unwrap_or_default();
                    Variant::from(items)
                }
                "IsStatusNotifierHostRegistered" => {
                    Variant::from(registered_for_property.load(Ordering::Relaxed))
                }
                "ProtocolVersion" => Variant::from(0i32),
                _ => Variant::from(0i32),
            },
        )
        // `RegistrationId` has no Drop implementation; dropping it does not
        // unregister the object, which stays exported for the host's lifetime.
        .build()?;

    Ok(())
}

/// The SNI protocol allows clients to pass either an object path (relative to
/// their own bus name) or a well-known bus name (item lives at the default path).
fn normalize_identifier(sender: Option<&str>, identifier: &str) -> ItemKey {
    if let Some(path) = identifier.strip_prefix('/') {
        return (sender.unwrap_or_default().to_string(), format!("/{path}"));
    }

    // A few exporters send "bus.name/object/path" as a single identifier.
    if let Some((service, path)) = identifier.split_once('/') {
        return (service.to_string(), format!("/{path}"));
    }

    (identifier.to_string(), "/StatusNotifierItem".to_string())
}

fn refresh(connection: &gio::DBusConnection) {
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
fn scan_bus_names(connection: &gio::DBusConnection) {
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

fn list_names(connection: &gio::DBusConnection) -> Vec<String> {
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
fn find_sni_path(connection: &gio::DBusConnection, service: &str) -> Option<String> {
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

/// Rebuilds the item snapshot advertised to the topbar.
fn publish_snapshot(connection: &gio::DBusConnection) {
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

fn read_item(connection: &gio::DBusConnection, service: &str, path: &str) -> Option<TrayItem> {
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
        icon_theme_path: properties
            .lookup("IconThemePath")
            .ok()
            .flatten()
            .unwrap_or_default(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifier_can_be_a_path_or_a_service_name() {
        assert_eq!(
            normalize_identifier(Some(":1.42"), "/org/foo/Tray"),
            (":1.42".to_string(), "/org/foo/Tray".to_string())
        );
        assert_eq!(
            normalize_identifier(Some(":1.42"), "org.foo.Tray"),
            (
                "org.foo.Tray".to_string(),
                "/StatusNotifierItem".to_string()
            )
        );
        assert_eq!(
            normalize_identifier(Some(":1.42"), "org.foo.Tray/Item"),
            ("org.foo.Tray".to_string(), "/Item".to_string())
        );
    }

    #[test]
    fn properties_dict_unwraps_variant_values() {
        // `Properties.GetAll` returns a{sv}; every value is wrapped in a `v`.
        let properties = VariantDict::new(None);
        properties.insert("Status", Variant::from("Active"));
        properties.insert("ItemIsMenu", Variant::from(true));
        let asv = properties.end();

        let parsed = VariantDict::new(Some(&asv));
        assert_eq!(
            parsed.lookup::<String>("Status").ok().flatten().as_deref(),
            Some("Active")
        );
        assert_eq!(
            parsed.lookup::<bool>("ItemIsMenu").ok().flatten(),
            Some(true)
        );
    }
}
