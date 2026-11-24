//! Data model for D-Bus menu items.

#[derive(Debug, Clone, PartialEq, Default)]
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
}
