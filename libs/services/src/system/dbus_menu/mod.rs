//! DBus Menu Protocol Implementation (com.canonical.dbusmenu)
//!
//! Discovery also falls back to AT-SPI accessibility menus, which are the
//! only menu source exposed by many Electron and Chromium applications.

mod atspi;
mod discovery;
mod discovery_helpers;
mod fetch;
mod models;
mod parse;

pub use discovery::find_app_menu_service;
pub use fetch::{
    activate_app_menu_item, activate_dbus_menu_item, fetch_app_menu, fetch_dbus_menu,
    fetch_dbus_menu_with_prepared_submenus,
};
pub use models::{DbusMenuItem, MenuActionRef};
