//! Right-edge gradient backdrop buffer and render element.

use common::constants::launcher::{BACKDROP_WIDTH_MIN, BACKDROP_WIDTH_RATIO};
use smithay::{
    backend::allocator::Fourcc,
    backend::renderer::{
        element::{
            memory::{MemoryRenderBuffer, MemoryRenderBufferRenderElement},
            Kind,
        },
        glow::GlowRenderer,
    },
    utils::{Buffer, Physical, Point, Rectangle, Scale, Transform},
};

use crate::modules::shell::launcher::backdrop::render_backdrop;
use crate::modules::render::CustomRenderElements;

use super::SwitcherOverlay;

/// Rebuilds the backdrop buffer when the size or fade tick changes.
///
/// Returns the physical backdrop width used to place the render element.
pub(super) fn update(
    overlay: &mut SwitcherOverlay,
    logical_w: f64,
    width: i32,
    height: i32,
    scale: Scale<f64>,
    presence: f64,
    presence_tick: u64,
) -> i32 {
    // Backdrop: same fade as the launcher, mirrored for the right dock.
    let gradient_w = (logical_w * BACKDROP_WIDTH_RATIO)
        .max(BACKDROP_WIDTH_MIN)
        .min(logical_w);
    let backdrop_w = ((gradient_w * scale.x).ceil() as i32).clamp(1, width);
    let backdrop_size = (backdrop_w, height);
    if overlay.backdrop_buffer.is_none()
        || overlay.backdrop_size != Some(backdrop_size)
        || overlay.backdrop_tick != Some(presence_tick)
    {
        if let Some(pixels) = render_backdrop(backdrop_w, height, scale, gradient_w, presence) {
            let mut buffer = MemoryRenderBuffer::new(
                Fourcc::Argb8888,
                backdrop_size,
                1,
                Transform::Normal,
                None,
            );
            {
                let mut context = buffer.render();
                let _ = context.draw(|dst| {
                    if dst.len() != pixels.len() {
                        return Err(());
                    }
                    dst.copy_from_slice(&pixels);
                    Ok(vec![Rectangle::<i32, Buffer>::from_size(
                        backdrop_size.into(),
                    )])
                });
            }
            overlay.backdrop_buffer = Some(buffer);
            overlay.backdrop_size = Some(backdrop_size);
            overlay.backdrop_tick = Some(presence_tick);
        }
    }

    backdrop_w
}

/// Builds the backdrop render element, placed behind the other overlay elements.
pub(super) fn elements(
    overlay: &SwitcherOverlay,
    backdrop_w: i32,
    width: i32,
    renderer: &mut GlowRenderer,
) -> Vec<CustomRenderElements> {
    let mut elements = Vec::new();
    if let Some(backdrop) = overlay.backdrop_buffer.as_ref() {
        let backdrop_x = (width - backdrop_w).max(0) as f64;
        if let Ok(elem) = MemoryRenderBufferRenderElement::from_buffer(
            renderer,
            Point::<f64, Physical>::from((backdrop_x, 0.0)),
            backdrop,
            None,
            None,
            None,
            Kind::Unspecified,
        ) {
            elements.push(CustomRenderElements::Memory(elem));
        }
    }
    elements
}
