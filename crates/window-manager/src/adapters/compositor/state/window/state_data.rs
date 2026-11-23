use smithay::{
    desktop::Window,
    utils::{Logical, Point, Size},
};

use super::animation::WindowAnimation;

/// Per-window persistent state attached to [`smithay::desktop::Window`]'s user_data.
#[derive(Default, Debug, Clone)]
pub(crate) struct WindowStateData {
    pub(crate) saved_location: Option<Point<i32, Logical>>,
    pub(crate) saved_size: Option<Size<i32, Logical>>,
    pub(crate) is_minimized: bool,
    pub(crate) has_opened: bool,
    pub(crate) animation: Option<WindowAnimation>,
}

/// Retrieves or initializes the window's state storage.
pub(crate) fn window_state(window: &Window) -> &std::sync::Mutex<WindowStateData> {
    window.user_data().get_or_insert(Default::default)
}

#[cfg(test)]
#[path = "../../../../tests/adapters/compositor/state/window/state_data.rs"]
mod tests;
