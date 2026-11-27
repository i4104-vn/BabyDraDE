//! Data model for D-Bus menu items.

/// Target of an AT-SPI `Action.DoAction` invocation.
///
/// `com.canonical.dbusmenu` items are activated by their numeric id; AT-SPI
/// items are addressed by the accessibility bus name and object path of the
/// accessible that implements `org.a11y.atspi.Action`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MenuActionRef {
    pub bus_name: String,
    pub object_path: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
#[non_exhaustive]
pub struct DbusMenuItem {
    pub id: i32,
    pub label: String,
    pub enabled: bool,
    pub visible: bool,
    pub is_separator: bool,
    /// True when the item is a menu container, even if its children are loaded lazily.
    pub has_submenu: bool,
    pub toggle_type: Option<String>,
    pub toggle_state: Option<i32>,
    pub children: Vec<DbusMenuItem>,
    /// Set only for AT-SPI backed items that can be activated with `DoAction`.
    pub action: Option<MenuActionRef>,
}
