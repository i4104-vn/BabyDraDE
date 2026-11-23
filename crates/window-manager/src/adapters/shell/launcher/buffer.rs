//! Off-screen pixel buffer helpers: only the dock region is allocated instead
//! of a full-screen surface.

use cairo::{Context, Format, ImageSurface};
use smithay::utils::{Buffer, Rectangle, Scale};

use common::constants::launcher::PANEL_FILLET_RADIUS;
use ui_kit::components::dock::DockLayout;

use super::{BufferRegion, LABEL_GAP, LABEL_MAX_WIDTH, SHADOW_PAD};

/// Physical region covered by the capsule, its shadow and the name label.
pub(crate) fn physical_region(
    panel: &DockLayout,
    scale: Scale<f64>,
    screen_w: i32,
    screen_h: i32,
) -> BufferRegion {
    let x0 = ((panel.x - SHADOW_PAD) * scale.x)
        .floor()
        .clamp(0.0, screen_w as f64) as i32;
    let y0 = ((panel.y - PANEL_FILLET_RADIUS - SHADOW_PAD) * scale.y)
        .floor()
        .clamp(0.0, screen_h as f64) as i32;
    let x1 = ((panel.x + panel.width + LABEL_GAP + LABEL_MAX_WIDTH + SHADOW_PAD) * scale.x)
        .ceil()
        .clamp(0.0, screen_w as f64) as i32;
    let y1 = ((panel.y + panel.height + PANEL_FILLET_RADIUS + SHADOW_PAD) * scale.y)
        .ceil()
        .clamp(0.0, screen_h as f64) as i32;

    (x0, y0, (x1 - x0).max(1), (y1 - y0).max(1))
}

/// Same backdrop mirrored for a right-side dock: darkest at the right edge.
pub(crate) fn render_right_backdrop_pixels(
    width: i32,
    height: i32,
    scale: Scale<f64>,
    gradient_w: f64,
    presence: f64,
) -> Option<Vec<u8>> {
    let mut surface = ImageSurface::create(Format::ARgb32, width, height).ok()?;
    {
        let context = Context::new(&surface).ok()?;
        context.scale(scale.x, scale.y);
        // The buffer only covers the backdrop strip, so the gradient must span
        // 0..gradient_w: dark at its right edge, fading out to the left.
        let gradient = cairo::LinearGradient::new(gradient_w, 0.0, 0.0, 0.0);
        gradient.add_color_stop_rgba(0.0, 0.0, 0.0, 0.0, 0.70 * presence);
        gradient.add_color_stop_rgba(0.30, 0.0, 0.0, 0.0, 0.45 * presence);
        gradient.add_color_stop_rgba(0.65, 0.0, 0.0, 0.0, 0.18 * presence);
        gradient.add_color_stop_rgba(1.0, 0.0, 0.0, 0.0, 0.0);
        context.set_source(&gradient).ok()?;
        context.paint().ok()?;
    }
    surface.flush();
    surface.data().ok().map(|data| data.to_vec())
}

/// Damage rectangle for a freshly written buffer.
pub(crate) fn buffer_damage(width: i32, height: i32) -> Vec<Rectangle<i32, Buffer>> {
    vec![Rectangle::<i32, Buffer>::from_size((width, height).into())]
}

/// Renders a full-height dark backdrop fading out smoothly to the right.
/// `presence` (0..=1) fades it together with the launcher capsule.
pub(crate) fn render_backdrop_pixels(
    width: i32,
    height: i32,
    scale: Scale<f64>,
    gradient_w: f64,
    presence: f64,
) -> Option<Vec<u8>> {
    let mut surface = ImageSurface::create(Format::ARgb32, width, height).ok()?;
    {
        let context = Context::new(&surface).ok()?;
        context.scale(scale.x, scale.y);
        let gradient = cairo::LinearGradient::new(0.0, 0.0, gradient_w, 0.0);
        gradient.add_color_stop_rgba(0.0, 0.0, 0.0, 0.0, 0.70 * presence);
        gradient.add_color_stop_rgba(0.30, 0.0, 0.0, 0.0, 0.45 * presence);
        gradient.add_color_stop_rgba(0.65, 0.0, 0.0, 0.0, 0.18 * presence);
        gradient.add_color_stop_rgba(1.0, 0.0, 0.0, 0.0, 0.0);
        context.set_source(&gradient).ok()?;
        context.paint().ok()?;
    }
    surface.flush();
    surface.data().ok().map(|data| data.to_vec())
}

#[cfg(test)]
#[path = "../../../tests/adapters/shell/launcher/buffer.rs"]
mod tests;
