//! Shared frame composition for all backends.
//!
//! Both the Winit and Udev backends build the same ordered list of render
//! elements; only output binding and presentation differ.

use smithay::{backend::renderer::glow::GlowRenderer, wayland::shell::wlr_layer::Layer};

use super::elements::{render_layer_surfaces, render_window, CustomRenderElements};

pub(crate) use crate::modules::render::models::FrameContext;

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
