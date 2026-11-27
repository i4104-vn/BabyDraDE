//! Window lookup hit testing.

use smithay::{
    desktop::Window,
    utils::{Logical, Point},
    wayland::shell::wlr_layer::Layer,
};

use crate::app::state::BabydraState;

impl BabydraState {
    pub(crate) fn window_under(
        &self,
        point: Point<f64, Logical>,
    ) -> Option<(Window, Point<i32, Logical>)> {
        // If an overlay or top layer surface (bars, popovers, menus) occludes this point,
        // no normal window can be under it.
        if let Some(layer) = self.layer_under(point) {
            if matches!(layer, Layer::Overlay | Layer::Top) {
                return None;
            }
        }

        // The compositor-owned frame sits just above the client surface.
        for window in self.windows.space.elements().rev() {
            if crate::modules::window::ops::is_window_exiting(window) {
                continue;
            }
            let Some(location) = self.windows.space.element_location(window) else {
                continue;
            };
            if crate::modules::window::hit_test::titlebar::hit_test(window, location, point)
                .is_some()
            {
                if self.is_occluded(window, point) {
                    break;
                }
                return Some((window.clone(), location));
            }
        }
        // 1. First check via Smithay's element_under (handles exact input regions)
        if let Some((window, _render_loc)) = self.windows.space.element_under(point) {
            if !crate::modules::window::ops::is_window_exiting(window) {
                if let Some(loc) = self.windows.space.element_location(window) {
                    return Some((window.clone(), loc));
                }
            }
        }

        // 2. Fallback: check window bounding box and geometry in space
        for window in self.windows.space.elements().rev() {
            if crate::modules::window::ops::is_window_exiting(window) {
                continue;
            }
            if let Some(bbox) = self.windows.space.element_bbox(window) {
                if bbox.to_f64().contains(point) {
                    if let Some(loc) = self.windows.space.element_location(window) {
                        return Some((window.clone(), loc));
                    }
                }
            }
            if let Some(geo) = self.windows.space.element_geometry(window) {
                if geo.to_f64().contains(point) {
                    if let Some(loc) = self.windows.space.element_location(window) {
                        return Some((window.clone(), loc));
                    }
                }
            }
        }

        None
    }
}
