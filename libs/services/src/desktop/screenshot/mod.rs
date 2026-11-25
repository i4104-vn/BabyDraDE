//! System I/O for screenshots.
//!
//! UI adapters compose images with Cairo/GTK and delegate file, notification,
//! and clipboard operations to this module. No GTK types cross this boundary.

mod handler;

pub use handler::{copy_to_clipboard, get_screenshot_path, notify, save_png};
