//! Foreground dock buffer: region allocation, animation cache and render element.

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

use crate::modules::shell::switcher::{
    render::{draw_switcher_foreground, SwitcherLayout},
    AltTabSwitcher,
};
use crate::modules::render::buffer::render_pixels;
use crate::modules::render::CustomRenderElements;

use super::SwitcherOverlay;
use crate::modules::shell::switcher::models::RenderKey;

/// Reallocates the foreground buffer when the capsule region moved, and
/// redraws it when the animation key changed.
pub(super) fn update(
    overlay: &mut SwitcherOverlay,
    switcher: &AltTabSwitcher,
    layout: &SwitcherLayout,
    region: (i32, i32, i32, i32),
    scale: Scale<f64>,
    slide_x: f64,
    presence: f64,
    render_key: RenderKey,
) {
    if overlay.fg_region != Some(region) {
        overlay.fg_buffer = Some(MemoryRenderBuffer::new(
            Fourcc::Argb8888,
            (region.2, region.3),
            1,
            Transform::Normal,
            None,
        ));
        overlay.fg_region = Some(region);
        overlay.last_rendered = None;
    }

    // Re-render the Cairo foreground while the selection tween is moving.
    // The microsecond tick gives the 120 fps loop a distinct frame every
    // time. This must also run with fewer than five apps: there the dock
    // items are static but the highlight frame glides between them, and
    // skipping these frames made it jump in one step.
    if overlay.last_rendered != Some(render_key) {
        let (origin_x, origin_y, buffer_w, buffer_h) = region;
        if let Some(fg_buf) = overlay.fg_buffer.as_mut() {
            if let Some(pixels) =
                render_pixels(buffer_w, buffer_h, scale, (origin_x, origin_y), |ctx| {
                    draw_switcher_foreground(ctx, switcher, layout, slide_x, presence)
                })
            {
                let mut context = fg_buf.render();
                let _ = context.draw(|buf| {
                    if buf.len() == pixels.len() {
                        buf.copy_from_slice(&pixels);
                        Ok(vec![Rectangle::<i32, Buffer>::from_size(
                            (buffer_w, buffer_h).into(),
                        )])
                    } else {
                        Err(())
                    }
                });
            }
        }

        overlay.last_rendered = Some(render_key);
    }
}

/// Builds the render element for the cached dock decorations.
pub(super) fn elements(
    overlay: &mut SwitcherOverlay,
    renderer: &mut GlowRenderer,
) -> Vec<CustomRenderElements> {
    let mut elements = Vec::new();
    if let (Some(fg_buf), Some((origin_x, origin_y, _, _))) =
        (overlay.fg_buffer.as_mut(), overlay.fg_region)
    {
        if let Ok(elem) = MemoryRenderBufferRenderElement::from_buffer(
            renderer,
            Point::<f64, Physical>::from((origin_x as f64, origin_y as f64)),
            fg_buf,
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

/// Physical region containing the right capsule, its shadow and the small
/// preview. Rounded outwards so anti-aliased edges are never clipped.
pub(super) fn overlay_region(
    layout: &SwitcherLayout,
    scale: Scale<f64>,
    screen_w: i32,
    screen_h: i32,
) -> (i32, i32, i32, i32) {
    let left = layout.preview.x.min(layout.panel.x) - 24.0;
    let right = (layout.panel.x + layout.panel.width).max(layout.preview.x + layout.preview.width)
        + 24.0
        + common::constants::launcher::PANEL_WIDTH;
    let top = layout.preview.y.min(layout.panel.y) - 24.0;
    let bottom =
        (layout.preview.y + layout.preview.height).max(layout.panel.y + layout.panel.height) + 24.0;

    let x0 = (left * scale.x).floor().clamp(0.0, screen_w as f64) as i32;
    let y0 = (top * scale.y).floor().clamp(0.0, screen_h as f64) as i32;
    let x1 = (right * scale.x).ceil().clamp(0.0, screen_w as f64) as i32;
    let y1 = (bottom * scale.y).ceil().clamp(0.0, screen_h as f64) as i32;

    (x0, y0, (x1 - x0).max(1), (y1 - y0).max(1))
}
