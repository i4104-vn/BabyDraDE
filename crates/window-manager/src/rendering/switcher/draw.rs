use cairo::{Context, FontSlant, FontWeight};

use crate::compositor::AltTabEntry;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Rect {
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) width: f64,
    pub(crate) height: f64,
}

impl Rect {
    pub(crate) fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width: width.max(1.0),
            height: height.max(1.0),
        }
    }

    pub(crate) fn center_x(&self) -> f64 {
        self.x + self.width / 2.0
    }

    #[allow(dead_code)]
    pub(crate) fn center_y(&self) -> f64 {
        self.y + self.height / 2.0
    }

    #[allow(dead_code)]
    pub(crate) fn inset(&self, dx: f64, dy: f64) -> Self {
        let w = (self.width - 2.0 * dx).max(1.0);
        let h = (self.height - 2.0 * dy).max(1.0);
        Self::new(self.x + dx, self.y + dy, w, h)
    }
}

#[allow(dead_code)]
pub(crate) fn lerp(from: f64, to: f64, progress: f64) -> f64 {
    from + (to - from) * progress
}

pub(crate) fn rounded_rect(
    context: &Context,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    radius: f64,
) {
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

pub(crate) fn draw_shadow(
    context: &Context,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    radius: f64,
    blur: f64,
    alpha: f64,
) -> Result<(), cairo::Error> {
    if alpha <= 0.01 {
        return Ok(());
    }
    let steps = 4;
    for i in (1..=steps).rev() {
        let spread = (i as f64 / steps as f64) * blur;
        let step_alpha = (alpha * 0.11) / (i as f64);
        rounded_rect(
            context,
            x - spread * 0.5,
            y - spread * 0.2 + spread * 0.5,
            width + spread,
            height + spread,
            radius + spread * 0.4,
        );
        context.set_source_rgba(0.0, 0.0, 0.0, step_alpha);
        context.fill()?;
    }
    Ok(())
}

pub(crate) fn draw_card_background(
    context: &Context,
    rect: Rect,
    radius: f64,
    alpha: f64,
) -> Result<(), cairo::Error> {
    if alpha <= 0.01 {
        return Ok(());
    }
    rounded_rect(context, rect.x, rect.y, rect.width, rect.height, radius);
    // Dark sleek translucent glass plate
    context.set_source_rgba(0.08, 0.09, 0.13, 0.90 * alpha);
    context.fill()?;
    Ok(())
}

pub(crate) fn draw_card_border(
    context: &Context,
    rect: Rect,
    radius: f64,
    is_active: bool,
    alpha: f64,
) -> Result<(), cairo::Error> {
    if alpha <= 0.01 {
        return Ok(());
    }
    rounded_rect(context, rect.x, rect.y, rect.width, rect.height, radius);
    if is_active {
        context.set_source_rgba(1.0, 1.0, 1.0, 0.35 * alpha);
        context.set_line_width(1.6);
    } else {
        context.set_source_rgba(1.0, 1.0, 1.0, 0.15 * alpha);
        context.set_line_width(1.0);
    }
    context.stroke()?;
    Ok(())
}

pub(crate) fn draw_dots_header(
    context: &Context,
    center_x: f64,
    center_y: f64,
    alpha: f64,
) -> Result<(), cairo::Error> {
    if alpha <= 0.01 {
        return Ok(());
    }
    let radius = 2.0;
    let spacing = 5.5;
    for &offset in &[-spacing, 0.0, spacing] {
        context.new_path();
        context.arc(
            center_x + offset,
            center_y,
            radius,
            0.0,
            2.0 * std::f64::consts::PI,
        );
        context.set_source_rgba(0.9, 0.9, 0.95, 0.55 * alpha);
        context.fill()?;
    }
    Ok(())
}

pub(crate) fn draw_stage_header_pill(
    context: &Context,
    entry: &AltTabEntry,
    center_x: f64,
    pill_y: f64,
    max_w: f64,
    alpha: f64,
) -> Result<(), cairo::Error> {
    if alpha <= 0.01 {
        return Ok(());
    }

    context.select_font_face("Sans", FontSlant::Normal, FontWeight::Bold);
    context.set_font_size(13.0);
    let app_text = fit_text(context, &entry.app_name, (max_w * 0.45).clamp(80.0, 200.0))?;
    let app_extents = context.text_extents(&app_text)?;

    context.select_font_face("Sans", FontSlant::Normal, FontWeight::Normal);
    context.set_font_size(12.0);
    let title_text = fit_text(context, &entry.title, (max_w * 0.45).clamp(80.0, 240.0))?;
    let title_extents = context.text_extents(&title_text)?;

    let icon_size = 18.0;
    let padding_x = 12.0;
    let gap = 8.0;
    let pill_w = padding_x * 2.0
        + icon_size
        + gap
        + app_extents.x_advance()
        + gap
        + title_extents.x_advance();
    let pill_h = 30.0;
    let pill_x = center_x - pill_w / 2.0;

    // Shadow
    draw_shadow(
        context,
        pill_x,
        pill_y,
        pill_w,
        pill_h,
        pill_h / 2.0,
        14.0,
        0.5 * alpha,
    )?;

    // Pill background
    rounded_rect(context, pill_x, pill_y, pill_w, pill_h, pill_h / 2.0);
    context.set_source_rgba(0.07, 0.08, 0.12, 0.88 * alpha);
    context.fill_preserve()?;
    context.set_source_rgba(1.0, 1.0, 1.0, 0.18 * alpha);
    context.set_line_width(1.0);
    context.stroke()?;

    // App icon inside pill
    let icon_x = pill_x + padding_x;
    let icon_y = pill_y + (pill_h - icon_size) / 2.0;
    draw_app_icon(
        context,
        entry,
        icon_x,
        icon_y,
        icon_size,
        app_color(&entry.app_name),
    )?;

    // App name
    let app_x = icon_x + icon_size + gap;
    let text_y = pill_y + pill_h / 2.0 + 4.5;
    context.select_font_face("Sans", FontSlant::Normal, FontWeight::Bold);
    context.set_font_size(13.0);
    draw_text(
        context,
        &app_text,
        app_x,
        text_y,
        (0.98, 0.99, 1.0, 0.96 * alpha),
    )?;

    // Title separator & title
    let title_x = app_x + app_extents.x_advance() + gap;
    context.select_font_face("Sans", FontSlant::Normal, FontWeight::Normal);
    context.set_font_size(12.0);
    draw_text(
        context,
        &title_text,
        title_x,
        text_y,
        (0.72, 0.77, 0.85, 0.85 * alpha),
    )?;

    Ok(())
}

pub(crate) fn draw_app_badge(
    context: &Context,
    entry: &AltTabEntry,
    x: f64,
    y: f64,
    size: f64,
    alpha: f64,
) -> Result<(), cairo::Error> {
    if alpha <= 0.01 {
        return Ok(());
    }
    let corner = size * 0.25;

    // Subtle drop shadow behind badge
    rounded_rect(context, x, y + 2.0, size, size, corner);
    context.set_source_rgba(0.0, 0.0, 0.0, 0.40 * alpha);
    context.fill()?;

    // Badge background with app color
    let color = app_color(&entry.app_name);
    rounded_rect(context, x, y, size, size, corner);
    context.set_source_rgba(color.0, color.1, color.2, 0.98 * alpha);
    context.fill_preserve()?;

    // Crisp white border
    context.set_source_rgba(1.0, 1.0, 1.0, 0.45 * alpha);
    context.set_line_width(1.2);
    context.stroke()?;

    // White bold initial
    context.select_font_face("Sans", FontSlant::Normal, FontWeight::Bold);
    context.set_font_size(size * 0.50);
    let initial = entry
        .app_name
        .chars()
        .next()
        .unwrap_or('A')
        .to_uppercase()
        .to_string();
    let extents = context.text_extents(&initial)?;
    let tx = x + (size - extents.x_advance()) / 2.0 - extents.x_bearing();
    let ty = y + (size - extents.height()) / 2.0 - extents.y_bearing();
    draw_text(context, &initial, tx, ty, (1.0, 1.0, 1.0, 0.98 * alpha))
}

pub(crate) fn draw_text(
    context: &Context,
    text: &str,
    x: f64,
    y: f64,
    color: (f64, f64, f64, f64),
) -> Result<(), cairo::Error> {
    context.set_source_rgba(color.0, color.1, color.2, color.3);
    context.move_to(x, y);
    context.show_text(text)
}

pub(crate) fn fit_text(
    context: &Context,
    text: &str,
    max_width: f64,
) -> Result<String, cairo::Error> {
    let clean = text.replace(['\n', '\r', '\t'], " ");
    if context.text_extents(&clean)?.x_advance() <= max_width {
        return Ok(clean);
    }

    let mut result = String::new();
    for character in clean.chars() {
        let candidate = format!("{result}{character}…");
        if context.text_extents(&candidate)?.x_advance() > max_width {
            break;
        }
        result.push(character);
    }
    result.push('…');
    Ok(result)
}

pub(crate) fn app_color(name: &str) -> (f64, f64, f64) {
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

pub(crate) fn draw_app_icon(
    context: &Context,
    entry: &AltTabEntry,
    x: f64,
    y: f64,
    size: f64,
    color: (f64, f64, f64),
) -> Result<(), cairo::Error> {
    rounded_rect(context, x, y, size, size, size * 0.23);
    context.set_source_rgba(color.0, color.1, color.2, 1.0);
    context.fill()?;

    context.select_font_face("Sans", FontSlant::Normal, FontWeight::Bold);
    context.set_font_size(size * 0.48);
    let initial = entry
        .app_name
        .chars()
        .next()
        .unwrap_or('A')
        .to_uppercase()
        .to_string();
    let extents = context.text_extents(&initial)?;
    let tx = x + (size - extents.x_advance()) / 2.0 - extents.x_bearing();
    let ty = y + (size - extents.height()) / 2.0 - extents.y_bearing();
    draw_text(context, &initial, tx, ty, (1.0, 1.0, 1.0, 0.98))
}
