use gio::glib::Variant;

use super::super::MenuActionRef;
use super::connection::{call, connect_atspi};
use super::ids::{AccessibleRef, ACTION_IFACE};

/// Activates an AT-SPI menu item through `org.a11y.atspi.Action.DoAction`.
pub(in crate::system::dbus_menu) fn activate_atspi_action(action: MenuActionRef) {
    std::thread::spawn(move || {
        let Some(conn) = connect_atspi() else {
            return;
        };
        let target = AccessibleRef {
            bus_name: action.bus_name,
            object_path: action.object_path,
        };
        let params = Variant::from((0i32,));
        let _ = call(&conn, &target, ACTION_IFACE, "DoAction", Some(&params));
    });
}
