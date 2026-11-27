//! Geometry, appearance and Cairo painting for one titlebar frame.

use cairo::Context;
use common::constants::{CORNER_RADIUS, TITLEBAR_HEIGHT};
use smithay::utils::{Logical, Point};

use crate::shared::types::geometry::Point as PurePoint;

pub(crate) use crate::modules::window::models::titlebar::{Controls, PaintStyle};
// Geometry and appearance data for one titlebar frame.
/// Converts a Smithay location to the pure point expected by the layout rule.
///
/// # Arguments
///
/// * `location` - Window location in logical coordinates.
///
/// # Returns
///
/// The equivalent framework-free point.
pub(crate) fn to_pure_location(location: Point<i32, Logical>) -> PurePoint<i32> {
    PurePoint::from((location.x, location.y))
}

// Titlebar background fill and bottom separator.
/// Paints the titlebar background and its adaptive bottom separator.
///
/// # Arguments
///
/// * `ctx` - Cairo context already scaled to logical coordinates.
/// * `width` - Titlebar width in logical pixels.
/// * `style` - Sampled color and translucency settings.
///
/// # Returns
///
/// `Some(())` when every Cairo operation succeeds.
pub(crate) fn draw_background(ctx: &Context, width: f64, style: &PaintStyle) -> Option<()> {
    let height = TITLEBAR_HEIGHT as f64;

    // Clear the surface to keep transparency outside rounded corners.
    ctx.set_operator(cairo::Operator::Clear);
    ctx.paint().ok()?;
    ctx.set_operator(cairo::Operator::Over);

    if !style.maximized {
        let r = CORNER_RADIUS.min(width / 2.0).min(height);
        ctx.new_sub_path();
        // Top-left rounded corner
        ctx.arc(
            r,
            r,
            r,
            std::f64::consts::PI,
            3.0 * std::f64::consts::FRAC_PI_2,
        );
        // Top-right rounded corner
        ctx.arc(width - r, r, r, -std::f64::consts::FRAC_PI_2, 0.0);
        // Bottom-right corner (straight)
        ctx.line_to(width, height);
        // Bottom-left corner (straight)
        ctx.line_to(0.0, height);
        ctx.close_path();
    } else {
        ctx.rectangle(0.0, 0.0, width, height);
    }

    if style.should_blur {
        ctx.set_source_rgba(
            style.color[0],
            style.color[1],
            style.color[2],
            style.opacity as f64,
        );
    } else {
        // No blur — fully opaque color matching the app background exactly.
        ctx.set_source_rgb(style.color[0], style.color[1], style.color[2]);
    }
    ctx.fill().ok()?;

    // Subtle 1px bottom separator adapting to dark/light background.
    let luminance = 0.299 * style.color[0] + 0.587 * style.color[1] + 0.114 * style.color[2];
    let (sep_r, sep_g, sep_b, sep_a) = if luminance > 0.5 {
        (0.0, 0.0, 0.0, 0.12)
    } else {
        (1.0, 1.0, 1.0, 0.08)
    };
    ctx.set_source_rgba(sep_r, sep_g, sep_b, sep_a);
    ctx.rectangle(0.0, height - 1.0, width, 1.0);
    ctx.fill().ok()?;

    Some(())
}

// Close, maximize, and minimize titlebar buttons.
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
pub(crate) fn draw_buttons(ctx: &Context, controls: &Controls, maximized: bool) -> Option<()> {
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
