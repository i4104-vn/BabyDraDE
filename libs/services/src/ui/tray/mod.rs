//! System Tray Support (`org.kde.StatusNotifierWatcher` / `org.kde.StatusNotifierItem`).
//!
//! The compositor's topbar acts as a Status Notifier Host: it owns the watcher
//! well-known name, accepts registrations from applications, discovers items
//! already present on the session bus, and keeps a snapshot of the active
//! items. GTK widgets only ever read that snapshot, so D-Bus traffic never
//! blocks the main loop.
//!
//! Status Notifier Host internals: watcher D-Bus object, item discovery and
//! property polling. Everything here runs on the tray host thread only.

mod discovery;
mod items;
mod registration;
mod watcher;

use std::collections::BTreeSet;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

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

/// One Status Notifier Item as displayed by a panel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrayItem {
    /// Bus name that owns the item.
    pub service: String,
    /// Object path exporting `org.kde.StatusNotifierItem`.
    pub path: String,
    /// Icon name (or absolute file path) advertised by the item.
    pub icon_name: String,
    /// Human readable tooltip / title.
    pub title: String,
    /// `Active`, `Attention` or `Passive`. Passive items are hidden.
    pub status: String,
    /// When true, a primary click opens the item's menu instead of activating it.
    pub item_is_menu: bool,
    /// Object path of the item's `com.canonical.dbusmenu` menu, when provided.
    pub menu_path: String,
}

/// Maximum time in milliseconds to wait for a tray item activation reply.
const ITEM_ACTIVATION_TIMEOUT_MS: i32 = 1000;

pub(super) type ItemKey = (String, String);

#[derive(Default)]
pub(super) struct HostState {
    /// Object path found for each probed bus name. `None` means "not an SNI item".
    pub(super) probed: std::collections::HashMap<String, Option<String>>,
    /// Items that explicitly called `RegisterStatusNotifierItem` on our watcher.
    pub(super) registered: BTreeSet<ItemKey>,
    /// Last full re-scan of the bus, used to catch late registrations.
    pub(super) last_probe: Option<std::time::Instant>,
}

pub(super) static HOST_STATE: OnceLock<Mutex<HostState>> = OnceLock::new();
static SNAPSHOT: OnceLock<Mutex<Vec<TrayItem>>> = OnceLock::new();

fn host_state() -> &'static Mutex<HostState> {
    HOST_STATE.get_or_init(|| Mutex::new(HostState::default()))
}

pub(super) fn snapshot() -> &'static Mutex<Vec<TrayItem>> {
    SNAPSHOT.get_or_init(|| Mutex::new(Vec::new()))
}

pub(super) use watcher::run;

/// Starts the Status Notifier Host in a dedicated thread with its own GLib main
/// context. Safe to call repeatedly; only one host is ever started.
///
/// Call this during shell startup, before applications are launched, so apps
/// that check for `org.kde.StatusNotifierWatcher` register their icons with us.
pub fn start_tray_host() {
    static STARTED: OnceLock<()> = OnceLock::new();
    STARTED.get_or_init(|| {
        if let Err(err) = std::thread::Builder::new()
            .name("babydra-tray-host".into())
            .spawn(run)
        {
            tracing::warn!("failed to start tray host thread: {err}");
        }
        // Touch the state early so registration callbacks never race initialization.
        let _ = host_state();
        let _ = snapshot();
    });
}

/// Returns the most recent snapshot of visible tray items. Non-blocking.
pub fn get_tray_items() -> Vec<TrayItem> {
    snapshot()
        .lock()
        .map(|items| items.clone())
        .unwrap_or_default()
}

/// Activates a tray item for a pointer button (`1` primary, `2` middle, `3` secondary).
///
/// The D-Bus call runs on a helper thread so callers can invoke it directly
/// from a GTK event handler.
pub fn activate_item(item: &TrayItem, x: i32, y: i32, button: u32) {
    let service = item.service.clone();
    let path = item.path.clone();
    let item_is_menu = item.item_is_menu;
    let menu_path = item.menu_path.clone();

    std::thread::spawn(move || {
        let Some(connection) =
            gio::bus_get_sync(gio::BusType::Session, gio::Cancellable::NONE).ok()
        else {
            return;
        };

        let method = match button {
            2 => "SecondaryActivate",
            3 => "ContextMenu",
            _ if item_is_menu && menu_path.is_empty() => "ContextMenu",
            _ => "Activate",
        };

        let parameters = gio::glib::Variant::from((x, y));
        if let Err(err) = connection.call_sync(
            Some(&service),
            &path,
            ITEM_IFACE,
            method,
            Some(&parameters),
            None,
            gio::DBusCallFlags::NONE,
            ITEM_ACTIVATION_TIMEOUT_MS,
            gio::Cancellable::NONE,
        ) {
            tracing::debug!("tray item {service}{path} did not handle {method}: {err}");
        }
    });
}

#[cfg(test)]
mod tests {
    use gio::glib::{Variant, VariantDict};

    use super::registration::{first_interface, normalize_identifier};

    #[test]
    fn empty_dbus_node_has_no_interface() {
        let node = gio::DBusNodeInfo::for_xml("<node/>").expect("valid empty node XML");
        let err = first_interface(&node).expect_err("empty node has no interfaces");
        assert!(!err.message().is_empty());
    }

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
