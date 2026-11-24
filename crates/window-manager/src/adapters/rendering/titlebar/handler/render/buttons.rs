//! Close, maximize, and minimize titlebar buttons.

use cairo::Context;

use super::super::models::Controls;

/// Draws the three titlebar buttons and their glyphs.
///
/// # Arguments
///
/// * `ctx` - Cairo context already scaled to logical coordinates.
/// * `controls` - Button centers and hover state for this frame.
/// * `maximized` - Whether the maximize button shows a restore glyph.
///
/// # Returns
///
/// `Some(())` when every Cairo operation succeeds.
pub(crate) fn draw(ctx: &Context, controls: &Controls, maximized: bool) -> Option<()> {
    let close_x = controls.close_x;
    let max_x = controls.max_x;
    let min_x = controls.min_x;
    let cy = controls.cy;

    // 1. Close button (Red)
    ctx.arc(close_x, cy, 6.0, 0.0, std::f64::consts::TAU);
    if controls.hover_close {
        ctx.set_source_rgb(1.0, 0.38, 0.35);
    } else {
        ctx.set_source_rgb(0.95, 0.32, 0.30);
    }
    ctx.fill().ok()?;

    if controls.in_cluster {
        let alpha = if controls.hover_close { 0.95 } else { 0.65 };
        ctx.set_source_rgba(0.30, 0.0, 0.0, alpha);
        ctx.set_line_width(1.2);
        ctx.set_line_cap(cairo::LineCap::Round);

        let offset = 2.4;
        ctx.move_to(close_x - offset, cy - offset);
        ctx.line_to(close_x + offset, cy + offset);
        ctx.stroke().ok()?;

        ctx.move_to(close_x + offset, cy - offset);
        ctx.line_to(close_x - offset, cy + offset);
        ctx.stroke().ok()?;
    }

    // 2. Maximize / Restore button (Yellow)
    ctx.arc(max_x, cy, 6.0, 0.0, std::f64::consts::TAU);
    if controls.hover_max {
        ctx.set_source_rgb(1.0, 0.78, 0.30);
    } else {
        ctx.set_source_rgb(0.98, 0.72, 0.25);
    }
    ctx.fill().ok()?;

    if controls.in_cluster {
        let alpha = if controls.hover_max { 0.95 } else { 0.65 };
        ctx.set_source_rgba(0.35, 0.20, 0.0, alpha);
        ctx.set_line_width(1.1);
        ctx.set_line_cap(cairo::LineCap::Square);
        ctx.set_line_join(cairo::LineJoin::Miter);

        if maximized {
            // Restore icon: two overlapping rectangles
            ctx.rectangle(max_x - 1.2, cy - 3.2, 4.4, 4.4);
            ctx.stroke().ok()?;
            ctx.rectangle(max_x - 3.2, cy - 1.2, 4.4, 4.4);
            ctx.stroke().ok()?;
        } else {
            // Maximize icon: single square
            let s = 5.0;
            ctx.rectangle(max_x - s / 2.0, cy - s / 2.0, s, s);
            ctx.stroke().ok()?;
        }
    }

    // 3. Minimize button (Green)
    ctx.arc(min_x, cy, 6.0, 0.0, std::f64::consts::TAU);
    if controls.hover_min {
        ctx.set_source_rgb(0.35, 0.88, 0.50);
    } else {
        ctx.set_source_rgb(0.30, 0.82, 0.45);
    }
    ctx.fill().ok()?;

    if controls.in_cluster {
        let alpha = if controls.hover_min { 0.95 } else { 0.65 };
        ctx.set_source_rgba(0.05, 0.25, 0.10, alpha);
        ctx.set_line_width(1.2);
        ctx.set_line_cap(cairo::LineCap::Round);

        let offset = 2.8;
        ctx.move_to(min_x - offset, cy);
        ctx.line_to(min_x + offset, cy);
        ctx.stroke().ok()?;
    }

    Some(())
}
