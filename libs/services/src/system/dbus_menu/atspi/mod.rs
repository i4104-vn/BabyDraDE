//! AT-SPI accessibility menu source.
//!
//! Electron (VS Code and its forks), Chromium and some Qt/GTK applications do
//! not export `com.canonical.dbusmenu`: their menu bar is only reachable via
//! the accessibility bus (`org.a11y.atspi.*`). This module speaks that D-Bus
//! API directly, so no additional crate or workspace manifest change is
//! required. It is used as a fallback after D-Bus menu discovery fails.

mod activation;
mod connection;
mod discovery;
mod ids;
mod menu;

#[cfg(test)]
mod tests;

pub(super) use activation::activate_atspi_action;
pub(super) use discovery::find_atspi_menubar;
pub(super) use ids::{atspi_bus_name, encode_atspi_source};
pub(super) use menu::fetch_atspi_menu;
