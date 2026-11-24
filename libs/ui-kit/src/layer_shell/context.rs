//! Configuration bundled for layer-shell window initialization.

use gtk4_layer_shell::{Edge, KeyboardMode, Layer};

/// Describes how a GTK window should be configured as a layer-shell surface.
///
/// The fields are consumed by [`init_layer_window`](super::init_layer_window)
/// to apply the layer, keyboard mode, exclusive zone, anchors, margin, and
/// namespace without passing a long argument list.
///
/// # Arguments
///
/// * `layer` - Shell layer the surface is placed on.
/// * `keyboard_mode` - Keyboard interaction mode for the surface.
/// * `exclusive_zone` - Exclusive zone requested from the compositor.
/// * `anchors` - Edges to anchor to and whether each anchor is enabled.
/// * `margin_bottom` - Bottom margin, applied only when greater than zero.
/// * `namespace` - Optional compositor namespace for the surface.
///
/// # Returns
///
/// A configuration value to pass by reference to [`init_layer_window`](super::init_layer_window).
#[derive(Debug, Clone, Copy)]
pub struct LayerWindowConfig<'a> {
    /// Shell layer the surface is placed on.
    pub layer: Layer,
    /// Keyboard interaction mode for the surface.
    pub keyboard_mode: KeyboardMode,
    /// Exclusive zone requested from the compositor (`-1` requests none).
    pub exclusive_zone: i32,
    /// Edges to anchor to; each pair holds an edge and whether it is enabled.
    pub anchors: &'a [(Edge, bool)],
    /// Bottom margin, applied only when greater than zero.
    pub margin_bottom: i32,
    /// Optional namespace used by the compositor for this surface.
    pub namespace: Option<&'a str>,
}
