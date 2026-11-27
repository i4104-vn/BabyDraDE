//! Window actions sent to the compositor's `wm` peer over the protocol bus.

use crate::protocol::{Request, WindowAction, WindowTarget};

use super::active::send_request;

/// Sends a window action through the protocol bus.
///
/// # Arguments
/// * `action` - Legacy action name (`close`, `minimize`, `maximize`, `fullscreen`).
pub fn send_window_action(action: &str) {
    let Some(action) = WindowAction::parse(action) else {
        return;
    };
    send_request(&Request::WindowAction { action });
}

/// Closes the window identified by `app_id` or `title`, or the focused one.
///
/// # Arguments
/// * `app_id` - Wayland app id to match, preferred when non-empty.
/// * `title` - Window title to match when `app_id` is empty.
pub fn close_window_by_app(app_id: &str, title: &str) {
    let target = if !app_id.is_empty() {
        WindowTarget::AppId(app_id.to_string())
    } else if !title.is_empty() {
        WindowTarget::Title(title.to_string())
    } else {
        WindowTarget::Focused
    };
    send_request(&Request::CloseTargeted { target });
}

/// Minimizes the currently focused window.
pub fn minimize_active_window() {
    send_request(&Request::WindowAction {
        action: WindowAction::Minimize,
    });
}

/// Maximizes the currently focused window.
pub fn maximize_active_window() {
    send_request(&Request::WindowAction {
        action: WindowAction::Maximize,
    });
}

/// Toggles fullscreen on the currently focused window.
pub fn toggle_fullscreen_active_window() {
    send_request(&Request::WindowAction {
        action: WindowAction::Fullscreen,
    });
}
