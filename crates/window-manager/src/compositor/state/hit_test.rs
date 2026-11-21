use smithay::{
    desktop::{layer_map_for_output, Window, WindowSurfaceType},
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Logical, Point},
    wayland::shell::wlr_layer::Layer,
};

use super::BabydraState;

impl BabydraState {
    /// Returns true when a window stacked above `window` has a client surface
    /// at `point`.
    fn is_occluded_by_window_above(&self, window: &Window, point: Point<f64, Logical>) -> bool {
        for other in self.space.elements().rev() {
            if other == window {
                return false;
            }
            if super::is_window_closing(other) {
                continue;
            }
            let Some(location) = self.space.element_location(other) else {
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
        // The compositor-owned frame sits just above the client surface.
        for window in self.space.elements().rev() {
            if super::is_window_closing(window) {
                continue;
            }
            let Some(location) = self.space.element_location(window) else {
                continue;
            };
            if crate::rendering::titlebar::hit_test(window, location, point).is_some() {
                if self.is_occluded_by_window_above(window, point) {
                    break;
                }
                return Some((window.clone(), location));
            }
        }
        // 1. First check via Smithay's element_under (handles exact input regions)
        if let Some((window, _render_loc)) = self.space.element_under(point) {
            if !super::is_window_closing(window) {
                if let Some(loc) = self.space.element_location(window) {
                    return Some((window.clone(), loc));
                }
            }
        }

        // 2. Fallback: check window bounding box and geometry in space
        for window in self.space.elements().rev() {
            if super::is_window_closing(window) {
                continue;
            }
            if let Some(bbox) = self.space.element_bbox(window) {
                if bbox.to_f64().contains(point) {
                    if let Some(loc) = self.space.element_location(window) {
                        return Some((window.clone(), loc));
                    }
                }
            }
            if let Some(geo) = self.space.element_geometry(window) {
                if geo.to_f64().contains(point) {
                    if let Some(loc) = self.space.element_location(window) {
                        return Some((window.clone(), loc));
                    }
                }
            }
        }

        None
    }

    pub(crate) fn surface_under(
        &self,
        position: Point<f64, Logical>,
    ) -> Option<(WlSurface, Point<f64, Logical>)> {
        if let Some(ref move_state) = self.move_state {
            if move_state.has_dragged {
                return None;
            }
        }

        // 1. Check Overlay and Top layer surfaces (bars, overlays, dynamic islands)
        if let Some(surface) = self.surface_under_layers(position, &[Layer::Overlay, Layer::Top]) {
            return Some(surface);
        }

        // 2. Check normal desktop windows
        if let Some((window, location)) = self.space.element_under(position) {
            if !super::is_window_closing(window) {
                if let Some((surface, point)) =
                    window.surface_under(position - location.to_f64(), WindowSurfaceType::ALL)
                {
                    return Some((surface, (point + location).to_f64()));
                }
            }
        }

        // 3. Check Bottom and Background layer surfaces (wallpapers, desktop widgets)
        self.surface_under_layers(position, &[Layer::Bottom, Layer::Background])
    }

    fn surface_under_layers(
        &self,
        position: Point<f64, Logical>,
        layers: &[Layer],
    ) -> Option<(WlSurface, Point<f64, Logical>)> {
        for output in self.space.outputs() {
            let layer_map = layer_map_for_output(output);
            for layer in layer_map.layers().rev() {
                if layers.contains(&layer.layer()) {
                    if let Some(geo) = layer_map.layer_geometry(layer) {
                        if geo.to_f64().contains(position) {
                            if let Some((surface, surface_location)) = layer
                                .surface_under(position - geo.loc.to_f64(), WindowSurfaceType::ALL)
                            {
                                return Some((surface, (surface_location + geo.loc).to_f64()));
                            }
                        }
                    }
                }
            }
        }

        None
    }

    pub(crate) fn layer_under(&self, position: Point<f64, Logical>) -> Option<Layer> {
        for output in self.space.outputs() {
            let layer_map = layer_map_for_output(output);
            for layer in layer_map.layers().rev() {
                if let Some(geo) = layer_map.layer_geometry(layer) {
                    if geo.to_f64().contains(position)
                        && layer
                            .surface_under(position - geo.loc.to_f64(), WindowSurfaceType::ALL)
                            .is_some()
                    {
                        return Some(layer.layer());
                    }
                }
            }
        }
        None
    }

    pub(crate) fn window_border_under(
        &self,
        point: Point<f64, Logical>,
    ) -> Option<(
        Window,
        Point<i32, Logical>,
        smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::ResizeEdge,
    )> {
        use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::{
            ResizeEdge, State,
        };

        use crate::enums::BORDER_MARGIN;

        for window in self.space.elements().rev() {
            if super::is_window_closing(window) {
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

            let Some(loc) = self.space.element_location(window) else {
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
                y - crate::rendering::titlebar::HEIGHT as f64
            } else {
                y
            };
            let total_h = if has_ssd {
                h + crate::rendering::titlebar::HEIGHT as f64
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
