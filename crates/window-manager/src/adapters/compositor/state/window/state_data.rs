use smithay::{
    desktop::Window,
    utils::{Logical, Point, Size},
};
use tracing::warn;

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

/// Locks per-window state without ever panicking on a poisoned mutex.
///
/// A panic in one frame must not take down the compositor on the next frame;
/// recovering the inner data keeps the last known window state usable.
pub(crate) fn lock_window_state(window: &Window) -> std::sync::MutexGuard<'_, WindowStateData> {
    match window_state(window).lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            warn!("recovering poisoned per-window state lock");
            poisoned.into_inner()
        }
    }
}
