//! Screenshot capture request state.

use std::sync::atomic::{AtomicBool, Ordering};

/// Shared flag set by the keymap thread and consumed by the render loops.
pub(crate) struct CaptureState {
    requested: AtomicBool,
}

impl CaptureState {
    pub(crate) const fn new() -> Self {
        Self {
            requested: AtomicBool::new(false),
        }
    }

    /// Requests a screenshot. Safe to call from any thread.
    pub(crate) fn request(&self) {
        self.requested.store(true, Ordering::SeqCst);
    }

    /// Consumes a pending screenshot request. Called once per rendered frame.
    pub(crate) fn take(&self) -> bool {
        self.requested.swap(false, Ordering::SeqCst)
    }
}
