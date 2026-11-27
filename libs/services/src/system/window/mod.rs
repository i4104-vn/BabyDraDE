//! Active window tracking and window actions for BabyDraDE features.
//!
//! Focused-window metadata is published as a retained `window.active` event on
//! the in-memory protocol bus. Window actions are requests to the compositor's
//! `wm` peer. Peers exchange framed JSON over a uid-scoped abstract Unix socket;
//! a router in the compositor owns delivery and retention. No files or queue
//! directories are involved.

mod actions;
mod active;
mod identity;

#[cfg(test)]
mod tests;

pub use actions::{
    close_window_by_app, maximize_active_window, minimize_active_window, send_window_action,
    toggle_fullscreen_active_window,
};
pub use active::get_active_window;
pub use common::protocol::ActiveWindowInfo;
pub use identity::{
    get_app_display_name, get_app_display_name_with_pid, get_process_app_name, get_title_app_part,
    get_window_app_candidates, get_window_desktop_entry_id, get_window_icon_id, is_shared_app_id,
};
