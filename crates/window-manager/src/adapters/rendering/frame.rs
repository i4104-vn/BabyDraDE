//! Shared frame composition for all backends.
//!
//! Both the Winit and Udev backends build the same ordered list of render
//! elements; only output binding and presentation differ.

use std::sync::{Arc, Mutex};

use smithay::{
    backend::renderer::glow::GlowRenderer,
    desktop::{Space, Window},
    output::Output,
    utils::Scale,
    wayland::shell::wlr_layer::Layer,
};

use crate::domain::WindowVisualState;

use super::{
    blur::BlurEngine,
    elements::{render_layer_surfaces, render_window, CustomRenderElements},
};

/// Inputs that are produced by the backend or shell before composition.
pub(crate) struct FrameContext<'a> {
    pub(crate) output: &'a Output,
    pub(crate) scale: Scale<f64>,
    pub(crate) cursor: Option<CustomRenderElements>,
    /// Shell elements rendered above layer-shell windows but below client windows.
    pub(crate) below_windows: Vec<CustomRenderElements>,
    /// Shell elements rendered above client windows (for example the switcher).
    pub(crate) above_windows: Vec<CustomRenderElements>,
    pub(crate) space: &'a Space<Window>,
    pub(crate) blur_engine: Option<&'a Arc<Mutex<BlurEngine>>>,
    pub(crate) pointer: smithay::utils::Point<f64, smithay::utils::Logical>,
    pub(crate) visuals: &'a dyn Fn(&Window) -> WindowVisualState,
}

/// Builds the frame element list in front-to-back order.
pub(crate) fn compose_frame(
    renderer: &mut GlowRenderer,
    ctx: FrameContext<'_>,
) -> Vec<CustomRenderElements> {
    let FrameContext {
        output,
        scale,
        cursor,
        below_windows,
        above_windows,
        space,
        blur_engine,
        pointer,
        visuals,
    } = ctx;
    let mut elements: Vec<CustomRenderElements> = Vec::new();

    // 1. The compositor-owned cursor is always on top.
    if let Some(cursor) = cursor {
        elements.push(cursor);
    }

    // 2. Top overlay layers (topbar, notifications).
    elements.extend(render_layer_surfaces(
        renderer,
        output,
        scale,
        &[Layer::Overlay, Layer::Top],
        blur_engine,
    ));

    // 3. Shell overlays below and above client windows.
    elements.extend(below_windows);
    elements.extend(above_windows);

    // 4. Client windows, front to back.
    for window in space.elements().rev() {
        if let Some(location) = space.element_location(window) {
            elements.extend(render_window(
                renderer,
                window,
                location,
                scale,
                blur_engine,
                pointer,
                visuals,
            ));
        }
    }

    // 5. Wallpaper and desktop layers at the bottom.
    elements.extend(render_layer_surfaces(
        renderer,
        output,
        scale,
        &[Layer::Bottom, Layer::Background],
        blur_engine,
    ));

    elements
}
