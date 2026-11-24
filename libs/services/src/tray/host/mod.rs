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

mod discovery;
mod items;
mod registration;
mod watcher;

pub(super) use watcher::run;

#[cfg(test)]
mod tests {
    use super::registration::normalize_identifier;
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
