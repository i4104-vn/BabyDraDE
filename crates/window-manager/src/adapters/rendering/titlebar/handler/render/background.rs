//! Titlebar background fill and bottom separator.

use cairo::Context;
use common::constants::{CORNER_RADIUS, TITLEBAR_HEIGHT};

use super::super::models::PaintStyle;

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
pub(crate) fn draw(ctx: &Context, width: f64, style: &PaintStyle) -> Option<()> {
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
