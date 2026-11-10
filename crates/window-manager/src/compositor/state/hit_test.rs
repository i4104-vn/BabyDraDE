use smithay::{
    desktop::{layer_map_for_output, Window, WindowSurfaceType},
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Logical, Point},
    wayland::shell::wlr_layer::Layer,
};

use super::BabydraState;

impl BabydraState {
    pub(crate) fn window_under(
        &self,
        point: Point<f64, Logical>,
    ) -> Option<(Window, Point<i32, Logical>)> {
        // 1. First check via Smithay's element_under (handles exact input regions)
        if let Some((window, _render_loc)) = self.space.element_under(point) {
            if let Some(loc) = self.space.element_location(window) {
                return Some((window.clone(), loc));
            }
        }

        // 2. Fallback: check window bounding box and geometry in space
        for window in self.space.elements().rev() {
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
            if let Some((surface, point)) =
                window.surface_under(position - location.to_f64(), WindowSurfaceType::ALL)
            {
                return Some((surface, (point + location).to_f64()));
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
}
