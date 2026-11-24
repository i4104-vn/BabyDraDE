//! Window hit testing.

use common::BORDER_MARGIN;
use smithay::{
    desktop::{Window, WindowSurfaceType},
    reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::{ResizeEdge, State},
    utils::{Logical, Point},
    wayland::shell::wlr_layer::Layer,
};

use super::super::BabydraState;

impl BabydraState {
    /// Returns true when a window stacked above `window` has a client surface
    /// at `point`.
    fn is_occluded_by_window_above(&self, window: &Window, point: Point<f64, Logical>) -> bool {
        for other in self.windows.space.elements().rev() {
            if other == window {
                return false;
            }
            if crate::adapters::compositor::state::is_window_exiting(other) {
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

    pub(crate) fn window_under(
        &self,
        point: Point<f64, Logical>,
    ) -> Option<(Window, Point<i32, Logical>)> {
        if let Some(layer) = self.layer_under(point) {
            if matches!(layer, Layer::Overlay | Layer::Top) {
                return None;
            }
        }

        // If an overlay or top layer surface (bars, popovers, menus) occludes this point,
        // no normal window can be under it.
        if let Some(layer) = self.layer_under(point) {
            if matches!(layer, Layer::Overlay | Layer::Top) {
                return None;
            }
        }

        // The compositor-owned frame sits just above the client surface.
        for window in self.windows.space.elements().rev() {
            if crate::adapters::compositor::state::is_window_exiting(window) {
                continue;
            }
            let Some(location) = self.windows.space.element_location(window) else {
                continue;
            };
            if crate::adapters::compositor::state::titlebar::hit_test(window, location, point)
                .is_some()
            {
                if self.is_occluded_by_window_above(window, point) {
                    break;
                }
                return Some((window.clone(), location));
            }
        }
        // 1. First check via Smithay's element_under (handles exact input regions)
        if let Some((window, _render_loc)) = self.windows.space.element_under(point) {
            if !crate::adapters::compositor::state::is_window_exiting(window) {
                if let Some(loc) = self.windows.space.element_location(window) {
                    return Some((window.clone(), loc));
                }
            }
        }

        // 2. Fallback: check window bounding box and geometry in space
        for window in self.windows.space.elements().rev() {
            if crate::adapters::compositor::state::is_window_exiting(window) {
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

    pub(crate) fn window_border_under(
        &self,
        point: Point<f64, Logical>,
    ) -> Option<(Window, Point<i32, Logical>, ResizeEdge)> {
        // If an overlay or top layer surface occludes this point, no border resize should occur.
        if let Some(layer) = self.layer_under(point) {
            if matches!(layer, Layer::Overlay | Layer::Top) {
                return None;
            }
        }

        for window in self.windows.space.elements().rev() {
            if crate::adapters::compositor::state::is_window_exiting(window) {
                continue;
            }

            let Some(toplevel) = window.toplevel() else {
                continue;
            };
            let current_state = toplevel.current_state();

            // Cannot resize maximized or fullscreen windows
            if current_state.states.contains(State::Maximized)
                || current_state.states.contains(State::Fullscreen)
            {
                continue;
            }

            let Some(loc) = self.windows.space.element_location(window) else {
                continue;
            };
            let geo = window.geometry();
            let w = geo.size.w as f64;
            let h = geo.size.h as f64;
            let x = loc.x as f64;
            let y = loc.y as f64;

            let has_ssd = current_state.decoration_mode
                == Some(
                    smithay::reexports::wayland_protocols::xdg::decoration::zv1::server::zxdg_toplevel_decoration_v1::Mode::ServerSide,
                );
            let top_y = if has_ssd {
                y - common::TITLEBAR_HEIGHT as f64
            } else {
                y
            };
            let total_h = if has_ssd {
                h + common::TITLEBAR_HEIGHT as f64
            } else {
                h
            };

            // Outer boundary with border margin
            if point.x < x - BORDER_MARGIN
                || point.x > x + w + BORDER_MARGIN
                || point.y < top_y - BORDER_MARGIN
                || point.y > top_y + total_h + BORDER_MARGIN
            {
                continue;
            }

            // Exclude titlebar buttons from top-edge resizing so clicks hit Close/Max/Min
            if has_ssd
                && point.x >= x + w - 85.0
                && point.x <= x + w
                && point.y >= top_y
                && point.y <= y
            {
                continue;
            }

            let on_left = point.x <= x + BORDER_MARGIN;
            let on_right = point.x >= x + w - BORDER_MARGIN;
            let on_top = point.y <= top_y + BORDER_MARGIN;
            let on_bottom = point.y >= top_y + total_h - BORDER_MARGIN;

            let edge = match (on_left, on_right, on_top, on_bottom) {
                (true, _, true, _) => ResizeEdge::TopLeft,
                (_, true, true, _) => ResizeEdge::TopRight,
                (true, _, _, true) => ResizeEdge::BottomLeft,
                (_, true, _, true) => ResizeEdge::BottomRight,
                (true, _, _, _) => ResizeEdge::Left,
                (_, true, _, _) => ResizeEdge::Right,
                (_, _, true, _) => ResizeEdge::Top,
                (_, _, _, true) => ResizeEdge::Bottom,
                _ => continue,
            };

            // Same occlusion rule as for titlebars: never start a resize on a
            // border that is hidden behind a higher window's surface.
            if self.is_occluded_by_window_above(window, point) {
                continue;
            }

            return Some((window.clone(), loc, edge));
        }

        None
    }
}
