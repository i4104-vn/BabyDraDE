// Inlined from window/state_data.rs.
use smithay::desktop::Window;
use tracing::warn;

pub(crate) use crate::modules::window::models::state::WindowStateData;

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
