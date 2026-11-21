//! Small, dependency-free Cairo primitives shared by shell surfaces.

use cairo::{Context, FontSlant, FontWeight};

/// RGBA tuple used by the drawing helpers.
type Color = (f64, f64, f64, f64);

/// Path for a rounded rectangle; callers decide how to fill or stroke it.
pub fn rounded_rect(context: &Context, x: f64, y: f64, width: f64, height: f64, radius: f64) {
    let radius = radius.min(width / 2.0).min(height / 2.0);
    context.new_path();
    context.move_to(x + radius, y);
    context.line_to(x + width - radius, y);
    context.arc(
        x + width - radius,
        y + radius,
        radius,
        -std::f64::consts::FRAC_PI_2,
        0.0,
    );
    context.line_to(x + width, y + height - radius);
    context.arc(
        x + width - radius,
        y + height - radius,
        radius,
        0.0,
        std::f64::consts::FRAC_PI_2,
    );
    context.line_to(x + radius, y + height);
    context.arc(
        x + radius,
        y + height - radius,
        radius,
        std::f64::consts::FRAC_PI_2,
        std::f64::consts::PI,
    );
    context.line_to(x, y + radius);
    context.arc(
        x + radius,
        y + radius,
        radius,
        std::f64::consts::PI,
        std::f64::consts::PI * 1.5,
    );
    context.close_path();
}

/// Fills a rounded shape and optionally adds the standard 1px hairline.
#[allow(clippy::too_many_arguments)]
pub fn paint_rounded(
    context: &Context,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    radius: f64,
    fill: Color,
    border: Option<Color>,
) -> Result<(), cairo::Error> {
    rounded_rect(context, x, y, width, height, radius);
    context.set_source_rgba(fill.0, fill.1, fill.2, fill.3);
    context.fill()?;

    if let Some(border) = border {
        rounded_rect(
            context,
            x + 0.5,
            y + 0.5,
            width - 1.0,
            height - 1.0,
            (radius - 0.5).max(0.0),
        );
        context.set_source_rgba(border.0, border.1, border.2, border.3);
        context.set_line_width(1.0);
        context.stroke()?;
    }
    Ok(())
}

/// Draws a text run at the given baseline.
pub fn draw_text(
    context: &Context,
    text: &str,
    x: f64,
    y: f64,
    color: Color,
) -> Result<(), cairo::Error> {
    context.set_source_rgba(color.0, color.1, color.2, color.3);
    context.move_to(x, y);
    context.show_text(text)
}

/// Selects the shared bold UI font at `size`.
pub fn set_bold_font(context: &Context, size: f64) {
    context.select_font_face("Sans", FontSlant::Normal, FontWeight::Bold);
    context.set_font_size(size);
}

/// Truncates `text` with an ellipsis so it fits within `max_width`.
pub fn fit_text(context: &Context, text: &str, max_width: f64) -> Result<String, cairo::Error> {
    if context.text_extents(text)?.x_advance() <= max_width {
        return Ok(text.to_string());
    }

    const ELLIPSIS: &str = "…";
    let ellipsis_width = context.text_extents(ELLIPSIS)?.x_advance();
    let mut fitted = String::new();
    for character in text.chars() {
        let mut candidate = fitted.clone();
        candidate.push(character);
        if context.text_extents(&candidate)?.x_advance() + ellipsis_width > max_width {
            break;
        }
        fitted = candidate;
    }
    fitted.push_str(ELLIPSIS);
    Ok(fitted)
}

/// Stable accent colour derived from an app name (FNV-1a hash → HSV).
pub fn app_color(name: &str) -> (f64, f64, f64) {
    let hash = name.bytes().fold(0x811c9dc5_u32, |hash, byte| {
        (hash ^ u32::from(byte)).wrapping_mul(0x01000193)
    });
    let hue = f64::from(hash % 360) / 360.0;
    let chroma = 0.50;
    let secondary = chroma * (1.0 - ((hue * 6.0) % 2.0 - 1.0).abs());
    let (red, green, blue) = match (hue * 6.0).floor() as u8 {
        0 => (chroma, secondary, 0.0),
        1 => (secondary, chroma, 0.0),
        2 => (0.0, chroma, secondary),
        3 => (0.0, secondary, chroma),
        4 => (secondary, 0.0, chroma),
        _ => (chroma, 0.0, secondary),
    };
    let offset = 0.28;
    (red + offset, green + offset, blue + offset)
}

/// Which screen edge a docked capsule attaches to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DockSide {
    Left,
    Right,
}

#[allow(clippy::too_many_arguments)]
pub fn docked_panel_path(
    context: &Context,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    corner_radius: f64,
    fillet_radius: f64,
    side: DockSide,
) -> Result<(), cairo::Error> {
    context.new_path();
    if side == DockSide::Right {
        // Draw the left-handed contour mirrored around the capsule centre.
        context.save()?;
        context.translate(x + width, 0.0);
        context.scale(-1.0, 1.0);
    }

    let r_convex = corner_radius.min(width / 2.0).min(height / 2.0);
    let r_concave = fillet_radius;
    let left = if side == DockSide::Right { 0.0 } else { x };

    context.move_to(left, y - r_concave);
    context.arc_negative(
        left + r_concave,
        y - r_concave,
        r_concave,
        std::f64::consts::PI,
        std::f64::consts::FRAC_PI_2,
    );
    context.line_to(left + width - r_convex, y);
    context.arc(
        left + width - r_convex,
        y + r_convex,
        r_convex,
        -std::f64::consts::FRAC_PI_2,
        0.0,
    );
    context.line_to(left + width, y + height - r_convex);
    context.arc(
        left + width - r_convex,
        y + height - r_convex,
        r_convex,
        0.0,
        std::f64::consts::FRAC_PI_2,
    );
    context.line_to(left + r_concave, y + height);
    context.arc_negative(
        left + r_concave,
        y + height + r_concave,
        r_concave,
        -std::f64::consts::FRAC_PI_2,
        -std::f64::consts::PI,
    );
    context.line_to(left, y - r_concave);
    context.close_path();

    if side == DockSide::Right {
        context.restore()?;
    }
    Ok(())
}

/// Draws a screen-edge capsule with concave fillets, soft shadow, surface fill
/// and a hairline border. Shared by the launcher (left) and switcher (right).
#[allow(clippy::too_many_arguments)]
pub fn draw_docked_panel(
    context: &Context,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    corner_radius: f64,
    fillet_radius: f64,
    surface: Color,
    border: Color,
    side: DockSide,
) -> Result<(), cairo::Error> {
    for (grow, alpha) in [(18.0, 0.06), (10.0, 0.09), (4.0, 0.13)] {
        docked_panel_path(
            context,
            x - grow,
            y - grow + 6.0,
            width + grow * 2.0,
            height + grow * 2.0,
            corner_radius + grow,
            fillet_radius + grow,
            side,
        )?;
        context.set_source_rgba(0.0, 0.0, 0.0, alpha);
        context.fill()?;
    }

    docked_panel_path(
        context,
        x,
        y,
        width,
        height,
        corner_radius,
        fillet_radius,
        side,
    )?;
    context.set_source_rgba(surface.0, surface.1, surface.2, surface.3);
    context.fill()?;

    docked_panel_path(
        context,
        x,
        y,
        width,
        height,
        corner_radius,
        fillet_radius,
        side,
    )?;
    context.set_source_rgba(border.0, border.1, border.2, border.3);
    context.set_line_width(1.0);
    context.stroke()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_color_is_stable_and_bounded() {
        let a = app_color("Firefox");
        let b = app_color("Firefox");
        assert_eq!(a, b);
        for channel in [a.0, a.1, a.2] {
            assert!((0.0..=1.0).contains(&channel));
        }
    }
}
