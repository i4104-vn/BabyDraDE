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

use crate::model::WindowVisualState;

use super::{
    blur::BlurEngine,
    elements::{render_layer_surfaces, render_window, CustomRenderElements},
};

/// Inputs that are produced by the backend or shell before composition.
pub struct FrameContext<'a> {
    pub output: &'a Output,
    pub scale: Scale<f64>,
    pub cursor: Option<CustomRenderElements>,
    /// Shell elements rendered above layer-shell windows but below client windows.
    pub below_windows: Vec<CustomRenderElements>,
    /// Shell elements rendered above client windows (for example the switcher).
    pub above_windows: Vec<CustomRenderElements>,
    pub space: &'a Space<Window>,
    pub blur_engine: Option<&'a Arc<Mutex<BlurEngine>>>,
    pub pointer: smithay::utils::Point<f64, smithay::utils::Logical>,
    pub visuals: &'a dyn Fn(&Window) -> WindowVisualState,
}

/// Builds the frame element list in front-to-back order.
pub fn compose_frame(
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
