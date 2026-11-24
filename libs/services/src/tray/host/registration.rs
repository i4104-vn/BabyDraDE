//! Tray host registration.

use super::watcher::{state, WATCHER_XML};
use super::*;

pub(super) fn request_watcher_name(connection: &gio::DBusConnection) -> bool {
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

/// Extracts the first D-Bus interface from parsed node info.
pub(super) fn first_interface(
    node: &gio::DBusNodeInfo,
) -> Result<gio::DBusInterfaceInfo, glib::Error> {
    node.interfaces().first().cloned().ok_or_else(|| {
        tracing::warn!("watcher XML defines no D-Bus interface");
        glib::Error::new(
            glib::FileError::Failed,
            "watcher XML defines no D-Bus interface",
        )
    })
}

pub(super) fn register_watcher(connection: &gio::DBusConnection) -> Result<(), glib::Error> {
    let node = gio::DBusNodeInfo::for_xml(WATCHER_XML)?;
    let interface = first_interface(&node)?;

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
pub(super) fn normalize_identifier(sender: Option<&str>, identifier: &str) -> ItemKey {
    if let Some(path) = identifier.strip_prefix('/') {
        return (sender.unwrap_or_default().to_string(), format!("/{path}"));
    }

    // A few exporters send "bus.name/object/path" as a single identifier.
    if let Some((service, path)) = identifier.split_once('/') {
        return (service.to_string(), format!("/{path}"));
    }

    (identifier.to_string(), "/StatusNotifierItem".to_string())
}
