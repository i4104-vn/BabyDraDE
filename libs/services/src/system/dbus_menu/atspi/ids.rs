pub(super) const A11Y_BUS_SERVICE: &str = "org.a11y.Bus";
pub(super) const A11Y_BUS_PATH: &str = "/org/a11y/bus";
pub(super) const A11Y_REGISTRY: &str = "org.a11y.atspi.Registry";
pub(super) const A11Y_DESKTOP_PATH: &str = "/org/a11y/atspi/accessible/root";
pub(super) const ACCESSIBLE_IFACE: &str = "org.a11y.atspi.Accessible";
pub(super) const ACTION_IFACE: &str = "org.a11y.atspi.Action";
pub(super) const PROPERTIES_IFACE: &str = "org.freedesktop.DBus.Properties";

pub(super) const STATE_ACTIVE: usize = 1;
pub(super) const STATE_DEFUNCT: usize = 6;
pub(super) const STATE_FOCUSED: usize = 12;
pub(super) const STATE_SENSITIVE: usize = 24;
pub(super) const STATE_HAS_POPUP: usize = 42;

pub(super) const CALL_TIMEOUT_MS: i32 = 150;
pub(super) const MAX_TREE_NODES: usize = 160;
pub(super) const MENU_TREE_DEPTH: usize = 6;

/// Marker used in the `(service, path)` pair returned by discovery so the
/// topbar can keep using the existing string-based menu handle.
const SOURCE_PREFIX: &str = "atspi:";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct AccessibleRef {
    pub(super) bus_name: String,
    pub(super) object_path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AccessibleRole {
    MenuBar,
    Menu,
    MenuItem,
    Separator,
    Other,
}

/// Encodes an AT-SPI target into the service/path pair used by the UI.
pub(in crate::system::dbus_menu) fn encode_atspi_source(
    bus_name: &str,
    object_path: &str,
) -> (String, String) {
    (
        format!("{SOURCE_PREFIX}{bus_name}"),
        object_path.to_string(),
    )
}

/// Returns the accessibility bus name when a service string is an AT-SPI source.
pub(in crate::system::dbus_menu) fn atspi_bus_name(service: &str) -> Option<&str> {
    service.strip_prefix(SOURCE_PREFIX)
}

/// AT-SPI state arrays are packed 32-bit words: word 0 holds states 0..31.
pub(super) fn state_has(states: &[u32], state: usize) -> bool {
    states
        .get(state / 32)
        .is_some_and(|word| word & (1u32 << (state % 32)) != 0)
}

/// Normalizes accessibility role names into the roles this module understands.
pub(super) fn classify_role(role: &str) -> AccessibleRole {
    let role = role.trim().to_ascii_lowercase();
    if role == "menubar" || role.contains("menu bar") {
        AccessibleRole::MenuBar
    } else if role.contains("separator") {
        AccessibleRole::Separator
    } else if role.contains("menu item") || role.contains("menuitem") {
        AccessibleRole::MenuItem
    } else if role == "menu" || role.contains("popup menu") || role.contains("section") {
        AccessibleRole::Menu
    } else {
        AccessibleRole::Other
    }
}
