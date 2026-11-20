//! Layer-shell initialization helper for the screenshot editor.

use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

/// Configures a window as a desktop shell layer.
#[allow(clippy::too_many_arguments)]
pub fn init_layer_window(
    window: &gtk4::ApplicationWindow,
    layer: Layer,
    kbd_mode: KeyboardMode,
    exclusive_zone: i32,
    anchors: &[(Edge, bool)],
    margin_bottom: i32,
    namespace: Option<&str>,
) {
    if !gtk4_layer_shell::is_supported() {
        return;
    }

    window.init_layer_shell();
    if let Some(ns) = namespace {
        window.set_namespace(Some(ns));
    }
    window.set_layer(layer);
    window.set_keyboard_mode(kbd_mode);
    window.set_exclusive_zone(exclusive_zone);
    for &(edge, anchor) in anchors {
        window.set_anchor(edge, anchor);
    }
    if margin_bottom > 0 {
        window.set_margin(Edge::Bottom, margin_bottom);
    }
}
