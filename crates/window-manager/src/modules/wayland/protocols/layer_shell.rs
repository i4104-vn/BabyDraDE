use smithay::{
    delegate_layer_shell,
    desktop::{layer_map_for_output, LayerSurface, WindowSurfaceType},
    output::Output,
    reexports::wayland_server::protocol::{wl_output::WlOutput, wl_surface::WlSurface},
    utils::SERIAL_COUNTER,
    wayland::shell::wlr_layer::{KeyboardInteractivity, Layer, WlrLayerShellHandler, WlrLayerShellState},
};
use tracing::{info, warn};

use crate::app::state::BabydraState;

impl WlrLayerShellHandler for BabydraState {
    fn shell_state(&mut self) -> &mut WlrLayerShellState {
        &mut self.wayland.wlr_layer_shell_state
    }

    fn new_layer_surface(
        &mut self,
        surface: smithay::wayland::shell::wlr_layer::LayerSurface,
        output: Option<WlOutput>,
        _layer: Layer,
        namespace: String,
    ) {
        let output = output
            .and_then(|o| Output::from_resource(&o))
            .or_else(|| self.windows.space.outputs().next().cloned());

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
pub(crate) fn handle_commit(state: &mut BabydraState, surface: &WlSurface) {
    let outputs: Vec<Output> = state.windows.space.outputs().cloned().collect();
    let mut should_focus = false;

    for output in &outputs {
        let mut layer_map = layer_map_for_output(output);
        if let Some(layer) = layer_map
            .layer_for_surface(surface, WindowSurfaceType::ALL)
            .cloned()
        {
            layer_map.arrange();
            if layer.layer_surface().has_pending_changes() {
                layer.layer_surface().send_configure();
            }

            // If the layer surface requests Exclusive keyboard interaction (e.g. screenshot editor, lock screen),
            // record that we need to assign keyboard focus.
            if layer.can_receive_keyboard_focus() {
                let is_exclusive = layer.cached_state().keyboard_interactivity == KeyboardInteractivity::Exclusive;
                if is_exclusive {
                    should_focus = true;
                }
            }
            break;
        }
    }

    if should_focus {
        if let Some(keyboard) = state.wayland.seat().get_keyboard() {
            if keyboard.current_focus().as_ref() != Some(surface) {
                let serial = SERIAL_COUNTER.next_serial();
                keyboard.set_focus(state, Some(surface.clone()), serial);
            }
        }
    }
}
