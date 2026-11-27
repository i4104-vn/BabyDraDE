use gio::glib::Variant;
use gio::DBusConnection;

use super::super::discovery::variant_string;
use super::ids::{
    AccessibleRef, A11Y_BUS_PATH, A11Y_BUS_SERVICE, ACCESSIBLE_IFACE, ACTION_IFACE,
    CALL_TIMEOUT_MS, PROPERTIES_IFACE,
};

pub(super) fn connect_atspi() -> Option<DBusConnection> {
    let session = gio::bus_get_sync(gio::BusType::Session, gio::Cancellable::NONE).ok()?;
    let reply = session
        .call_sync(
            Some(A11Y_BUS_SERVICE),
            A11Y_BUS_PATH,
            A11Y_BUS_SERVICE,
            "GetAddress",
            None,
            None,
            gio::DBusCallFlags::NONE,
            CALL_TIMEOUT_MS,
            gio::Cancellable::NONE,
        )
        .ok()?;
    let address_value = reply.child_value(0);
    let address = address_value.str()?.to_string();
    DBusConnection::for_address_sync(
        &address,
        gio::DBusConnectionFlags::NONE,
        None,
        gio::Cancellable::NONE,
    )
    .ok()
}

pub(super) fn call(
    conn: &DBusConnection,
    target: &AccessibleRef,
    interface: &str,
    method: &str,
    params: Option<&Variant>,
) -> Option<Variant> {
    conn.call_sync(
        Some(&target.bus_name),
        &target.object_path,
        interface,
        method,
        params,
        None,
        gio::DBusCallFlags::NONE,
        CALL_TIMEOUT_MS,
        gio::Cancellable::NONE,
    )
    .ok()
}

pub(super) fn property_string(
    conn: &DBusConnection,
    target: &AccessibleRef,
    name: &str,
) -> Option<String> {
    let params = Variant::from((ACCESSIBLE_IFACE, name));
    let reply = call(conn, target, PROPERTIES_IFACE, "Get", Some(&params))?;
    reply.child_value(0).str().map(str::to_string)
}

pub(super) fn role_name(conn: &DBusConnection, target: &AccessibleRef) -> String {
    call(conn, target, ACCESSIBLE_IFACE, "GetRoleName", None)
        .and_then(|reply| reply.child_value(0).str().map(str::to_string))
        .unwrap_or_default()
}

pub(super) fn state_set(conn: &DBusConnection, target: &AccessibleRef) -> Vec<u32> {
    let Some(reply) = call(conn, target, ACCESSIBLE_IFACE, "GetState", None) else {
        return Vec::new();
    };
    let states = reply.child_value(0);
    (0..states.n_children())
        .filter_map(|index| states.child_value(index).get::<u32>())
        .collect()
}

pub(super) fn accessible_children(
    conn: &DBusConnection,
    target: &AccessibleRef,
) -> Vec<AccessibleRef> {
    let Some(reply) = call(conn, target, ACCESSIBLE_IFACE, "GetChildren", None) else {
        return Vec::new();
    };
    let children = reply.child_value(0);
    (0..children.n_children())
        .filter_map(|index| {
            let child = children.child_value(index);
            let bus_name = variant_string(&child.child_value(0))?;
            let object_path = variant_string(&child.child_value(1))?;
            Some(AccessibleRef {
                bus_name,
                object_path,
            })
        })
        .collect()
}

pub(super) fn has_action(conn: &DBusConnection, target: &AccessibleRef) -> bool {
    let Some(reply) = call(conn, target, ACCESSIBLE_IFACE, "GetInterfaces", None) else {
        return false;
    };
    let interfaces = reply.child_value(0);
    (0..interfaces.n_children()).any(|index| {
        interfaces
            .child_value(index)
            .str()
            .is_some_and(|name| name == ACTION_IFACE)
    })
}
