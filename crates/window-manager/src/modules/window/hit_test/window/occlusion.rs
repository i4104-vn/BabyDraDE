//! Occlusion checks for stacked windows.

use smithay::{
    desktop::{Window, WindowSurfaceType},
    utils::{Logical, Point},
};

use crate::app::state::BabydraState;

impl BabydraState {
    /// Returns true when a window stacked above `window` has a client surface
    /// at `point`.
    pub(super) fn is_occluded(&self, window: &Window, point: Point<f64, Logical>) -> bool {
        for other in self.windows.space.elements().rev() {
            if other == window {
                return false;
            }
            if crate::modules::window::ops::is_window_exiting(other) {
                continue;
            }
            let Some(location) = self.windows.space.element_location(other) else {
                continue;
            };
            // Match `Space::element_under`: input regions are relative to the
            // surface render location, not the element's mapped location.
            let render_location = location - other.geometry().loc;
            if other
                .surface_under(point - render_location.to_f64(), WindowSurfaceType::ALL)
                .is_some()
            {
                return true;
            }
        }
        false
    }
}
