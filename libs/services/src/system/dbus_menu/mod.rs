//! DBus Menu Protocol Implementation (com.canonical.dbusmenu)

mod discovery;
mod discovery_helpers;
mod fetch;
mod models;
mod parse;

pub use discovery::find_app_menu_service;
pub use fetch::{activate_dbus_menu_item, fetch_dbus_menu, fetch_dbus_menu_with_prepared_submenus};
pub use models::DbusMenuItem;
