//! Window facts shared across layers (pure data).
//!
//! The adapter extracts protocol data into these types; the domain never
//! touches a `Window` handle, config file, cache or system service.

use smithay::utils::{Logical, Rectangle};

/// Per-frame visual values the renderer applies to one window.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct WindowVisualState {
    pub(crate) active: bool,
    pub(crate) scale_factor: f64,
    pub(crate) alpha_factor: f32,
    pub(crate) target_rect: Option<Rectangle<f64, Logical>>,
}

impl WindowVisualState {
    /// Returns the neutral visual state with no animation applied.
    pub(crate) fn idle() -> Self {
        Self {
            active: false,
            scale_factor: 1.0,
            alpha_factor: 1.0,
            target_rect: None,
        }
    }
}

/// Identity and display name extracted from one Wayland toplevel.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct WindowMetadata {
    pub(crate) app_id: String,
    pub(crate) title: String,
    pub(crate) app_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_visual_state_is_neutral() {
        let state = WindowVisualState::idle();
        assert!(!state.active);
        assert_eq!(state.scale_factor, 1.0);
        assert_eq!(state.alpha_factor, 1.0);
        assert!(state.target_rect.is_none());
    }
}
