//! Applies layer-shell configuration to GTK windows.

use gtk4_layer_shell::{Edge, LayerShell};

use super::LayerWindowConfig;

/// Configures a GTK window as a layer-shell surface.
///
/// Does nothing when the compositor does not support the layer-shell protocol.
///
/// # Arguments
///
/// * `window` - Application window to configure as a layer-shell surface.
/// * `config` - Layer, keyboard mode, exclusive zone, anchors, margin, and namespace to apply.
///
/// # Returns
///
/// Nothing. The window is configured in place, or left untouched if layer shell
/// is unsupported.
pub fn init_layer_window(window: &gtk4::ApplicationWindow, config: &LayerWindowConfig<'_>) {
    if !gtk4_layer_shell::is_supported() {
        return;
    }

    window.init_layer_shell();
    if let Some(namespace) = config.namespace {
        window.set_namespace(Some(namespace));
    }
    window.set_layer(config.layer);
    window.set_keyboard_mode(config.keyboard_mode);
    window.set_exclusive_zone(config.exclusive_zone);
    for &(edge, anchor) in config.anchors {
        window.set_anchor(edge, anchor);
    }
    if config.margin_bottom > 0 {
        window.set_margin(Edge::Bottom, config.margin_bottom);
    }
}
