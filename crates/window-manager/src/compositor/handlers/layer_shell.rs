use smithay::{
    delegate_layer_shell,
    desktop::{layer_map_for_output, LayerSurface, WindowSurfaceType},
    output::Output,
    reexports::wayland_server::protocol::{wl_output::WlOutput, wl_surface::WlSurface},
    wayland::shell::wlr_layer::{
        Layer, LayerSurface as WlrLayerSurface, WlrLayerShellHandler, WlrLayerShellState,
    },
};
use tracing::{info, warn};

use crate::compositor::state::BabydraState;

impl WlrLayerShellHandler for BabydraState {
    fn shell_state(&mut self) -> &mut WlrLayerShellState {
        &mut self.wlr_layer_shell_state
    }

    fn new_layer_surface(
        &mut self,
        surface: WlrLayerSurface,
        output: Option<WlOutput>,
        _layer: Layer,
        namespace: String,
    ) {
        let output = output
            .and_then(|o| Output::from_resource(&o))
            .or_else(|| self.space.outputs().next().cloned());

        if let Some(output) = output {
            info!(
                "LayerShell: Mapping new layer surface '{}' onto output '{}'",
                namespace,
                output.name()
            );
            let mut layer_map = layer_map_for_output(&output);
            let layer_surface = LayerSurface::new(surface, namespace);
            let _ = layer_map.map_layer(&layer_surface);
            layer_map.arrange();
            layer_surface.layer_surface().send_configure();
        } else {
            warn!(
                "LayerShell: No output available to map layer surface '{}'!",
                namespace
            );
        }
    }
}

delegate_layer_shell!(BabydraState);

/// Handles surface commits for layer shell surfaces across all outputs.
pub fn handle_commit(state: &BabydraState, surface: &WlSurface) {
    for output in state.space.outputs() {
        let mut layer_map = layer_map_for_output(output);
        if let Some(layer) = layer_map
            .layer_for_surface(surface, WindowSurfaceType::ALL)
            .cloned()
        {
            layer_map.arrange();
            if layer.layer_surface().has_pending_changes() {
                layer.layer_surface().send_configure();
            }
            return;
        }
    }
}
