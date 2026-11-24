//! DBus Menu Protocol Implementation (com.canonical.dbusmenu)

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

mod discovery;
mod discovery_helpers;
mod fetch;
mod parse;

pub use discovery::find_app_menu_service;
pub use fetch::{
    activate_dbus_menu_item, fetch_dbus_menu, fetch_dbus_menu_with_prepared_submenus,
};
