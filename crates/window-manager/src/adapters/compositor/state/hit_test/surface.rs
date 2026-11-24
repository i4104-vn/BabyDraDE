//! Surface and layer-shell hit testing.

use smithay::{
    desktop::{layer_map_for_output, WindowSurfaceType},
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Logical, Point},
    wayland::shell::wlr_layer::Layer,
};

use super::super::BabydraState;

impl BabydraState {
    pub(crate) fn surface_under(
        &self,
        position: Point<f64, Logical>,
    ) -> Option<(WlSurface, Point<f64, Logical>)> {
        if let Some(ref move_state) = self.input.move_state {
            if move_state.has_dragged {
                return None;
            }
        }

        // 1. Check Overlay and Top layer surfaces (bars, overlays, dynamic islands)
        if let Some(surface) = self.surface_under_layers(position, &[Layer::Overlay, Layer::Top]) {
            return Some(surface);
        }

        // 2. Check normal desktop windows
        if let Some((window, location)) = self.windows.space.element_under(position) {
            if !crate::adapters::compositor::state::is_window_exiting(window) {
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
        for output in self.windows.space.outputs() {
            let layer_map = layer_map_for_output(output);
            for layer in layer_map.layers().rev() {
                if layers.contains(&layer.layer()) {
                    if let Some(geo) = layer_map.layer_geometry(layer) {
                        let mut bbox = layer.bbox_with_popups();
                        bbox.loc += geo.loc;
                        if bbox.to_f64().contains(position) {
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
        for output in self.windows.space.outputs() {
            let layer_map = layer_map_for_output(output);
            for layer in layer_map.layers().rev() {
                if let Some(geo) = layer_map.layer_geometry(layer) {
                    let mut bbox = layer.bbox_with_popups();
                    bbox.loc += geo.loc;
                    if bbox.to_f64().contains(position)
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
