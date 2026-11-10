use std::time::Duration;

use cairo::{Context, FontSlant, FontWeight, Format, ImageSurface, Operator};
use smithay::{
    backend::{
        allocator::Fourcc,
        renderer::{
            element::{
                memory::{MemoryRenderBuffer, MemoryRenderBufferRenderElement},
                Kind,
            },
            glow::GlowRenderer,
        },
    },
    utils::{Buffer, Physical, Point, Rectangle, Transform},
};

use crate::compositor::{AltTabEntry, AltTabSwitcher};

use super::CustomRenderElements;

const ANIMATION_DURATION: Duration = Duration::from_millis(220);

#[derive(Default)]
pub struct SwitcherOverlay {
    buffer: Option<MemoryRenderBuffer>,
    size: (i32, i32),
    last_rendered: Option<(std::time::Instant, usize)>,
}

impl SwitcherOverlay {
    pub(crate) fn element(
        &mut self,
        renderer: &mut GlowRenderer,
        switcher: Option<&AltTabSwitcher>,
        size: (i32, i32),
    ) -> Option<CustomRenderElements> {
        let Some(switcher) = switcher else {
            self.last_rendered = None;
            return None;
        };
        let (width, height) = (size.0.max(1), size.1.max(1));
        if self.size != (width, height) {
            self.buffer = Some(MemoryRenderBuffer::new(
                Fourcc::Argb8888,
                (width, height),
                1,
                Transform::Normal,
                None,
            ));
            self.size = (width, height);
            self.last_rendered = None;
        }

        let buffer = self.buffer.as_mut()?;
        let animation_active = switcher.transition_started.elapsed() < ANIMATION_DURATION;
        let render_key = (switcher.transition_started, switcher.selected_index);
        if animation_active || self.last_rendered != Some(render_key) {
            let pixels = render_pixels(width, height, switcher)?;
            let mut context = buffer.render();
            let _ = context.draw(|buffer| {
                if buffer.len() != pixels.len() {
                    return Err(());
                }
                buffer.copy_from_slice(&pixels);
                Ok(vec![Rectangle::<i32, Buffer>::from_size(
                    (width, height).into(),
                )])
            });
            drop(context);
            self.last_rendered = Some(render_key);
        }

        MemoryRenderBufferRenderElement::from_buffer(
            renderer,
            Point::<f64, Physical>::from((0.0, 0.0)),
            buffer,
            None,
            None,
            None,
            Kind::Unspecified,
        )
        .ok()
        .map(CustomRenderElements::Memory)
    }
}

fn render_pixels(width: i32, height: i32, switcher: &AltTabSwitcher) -> Option<Vec<u8>> {
    let mut surface = ImageSurface::create(Format::ARgb32, width, height).ok()?;
    {
        let context = Context::new(&surface).ok()?;
        draw_switcher(&context, width as f64, height as f64, switcher).ok()?;
    }
    surface.flush();
    surface.data().ok().map(|data| data.to_vec())
}

fn draw_switcher(
    context: &Context,
    width: f64,
    height: f64,
    switcher: &AltTabSwitcher,
) -> Result<(), cairo::Error> {
    if switcher.entries.is_empty() {
        return Ok(());
    }

    let progress: f64 = (switcher.transition_started.elapsed().as_secs_f64()
        / ANIMATION_DURATION.as_secs_f64())
    .clamp(0.0, 1.0);
    let eased = 1.0f64 - (1.0f64 - progress).powi(3);

    context.set_operator(Operator::Source);
    context.set_source_rgba(0.0, 0.0, 0.0, 0.0);
    context.paint()?;
    context.set_operator(Operator::Over);

    context.set_source_rgba(0.015, 0.02, 0.04, 0.76);
    context.paint()?;

    let panel_width = 272.0_f64.min((width - 40.0).max(210.0));
    let panel_height = (height - 48.0).clamp(170.0, 700.0);
    let panel_x = 24.0;
    let panel_y = ((height - panel_height) / 2.0).max(12.0);
    rounded_rect(context, panel_x, panel_y, panel_width, panel_height, 22.0);
    context.set_source_rgba(0.055, 0.07, 0.11, 0.97);
    context.fill_preserve()?;
    context.set_source_rgba(1.0, 1.0, 1.0, 0.12);
    context.set_line_width(1.0);
    context.stroke()?;

    context.select_font_face("Sans", FontSlant::Normal, FontWeight::Bold);
    context.set_font_size(15.0);
    draw_text(
        context,
        "Open apps",
        panel_x + 20.0,
        panel_y + 31.0,
        (0.94, 0.96, 1.0, 1.0),
    )?;
    context.select_font_face("Sans", FontSlant::Normal, FontWeight::Normal);
    context.set_font_size(11.0);
    draw_text(
        context,
        &format!("{} windows", switcher.entries.len()),
        panel_x + 20.0,
        panel_y + 49.0,
        (0.56, 0.62, 0.72, 1.0),
    )?;

    let row_height = 58.0;
    let list_top = panel_y + 64.0;
    let visible_count = (((panel_height - 78.0) / row_height).floor() as usize).max(1);
    let first_visible = switcher
        .selected_index
        .saturating_sub(visible_count / 2)
        .min(switcher.entries.len().saturating_sub(visible_count));
    let last_visible = (first_visible + visible_count).min(switcher.entries.len());
    let visible_row = |index: usize| -> Option<f64> {
        (index >= first_visible && index < last_visible)
            .then_some(list_top + (index - first_visible) as f64 * row_height)
    };

    for (index, entry) in switcher
        .entries
        .iter()
        .enumerate()
        .take(last_visible)
        .skip(first_visible)
    {
        let row_y = visible_row(index).unwrap_or(list_top);
        let selected = index == switcher.selected_index;
        if selected {
            rounded_rect(
                context,
                panel_x + 9.0,
                row_y,
                panel_width - 18.0,
                50.0,
                13.0,
            );
            context.set_source_rgba(0.30, 0.48, 0.82, 0.38);
            context.fill()?;
            rounded_rect(
                context,
                panel_x + 9.0,
                row_y,
                panel_width - 18.0,
                50.0,
                13.0,
            );
            context.set_source_rgba(0.57, 0.70, 1.0, 0.30);
            context.set_line_width(1.0);
            context.stroke()?;
        }

        let icon_color = app_color(&entry.app_name);
        draw_app_icon(
            context,
            entry,
            panel_x + 19.0,
            row_y + 8.0,
            34.0,
            icon_color,
        )?;

        context.select_font_face("Sans", FontSlant::Normal, FontWeight::Bold);
        context.set_font_size(13.0);
        let app_name = fit_text(context, &entry.app_name, panel_width - 84.0)?;
        draw_text(
            context,
            &app_name,
            panel_x + 64.0,
            row_y + 22.0,
            if selected {
                (0.98, 0.99, 1.0, 1.0)
            } else {
                (0.85, 0.88, 0.93, 1.0)
            },
        )?;

        context.select_font_face("Sans", FontSlant::Normal, FontWeight::Normal);
        context.set_font_size(10.5);
        let title = fit_text(context, &entry.title, panel_width - 84.0)?;
        draw_text(
            context,
            &title,
            panel_x + 64.0,
            row_y + 38.0,
            (0.59, 0.65, 0.74, 1.0),
        )?;
    }

    let Some(entry) = switcher.entries.get(switcher.selected_index) else {
        return Ok(());
    };
    let target_width = (width * 0.48).clamp(300.0, 520.0).min(width - 48.0);
    let target_height = (height * 0.42).clamp(220.0, 330.0).min(height - 48.0);
    let target = Rect {
        x: (width - target_width) / 2.0,
        y: (height - target_height) / 2.0,
        width: target_width,
        height: target_height,
    };
    let source = visible_row(switcher.selected_index)
        .map(|row_y| Rect {
            x: panel_x + 16.0,
            y: row_y + 4.0,
            width: 46.0,
            height: 46.0,
        })
        .unwrap_or(Rect {
            x: panel_x + 16.0,
            y: panel_y + 72.0,
            width: 46.0,
            height: 46.0,
        });
    let card = Rect {
        x: lerp(source.x, target.x, eased),
        y: lerp(source.y, target.y, eased),
        width: lerp(source.width, target.width, eased),
        height: lerp(source.height, target.height, eased),
    };

    draw_selected_card(context, entry, card, eased)?;
    Ok(())
}

fn draw_selected_card(
    context: &Context,
    entry: &AltTabEntry,
    rect: Rect,
    progress: f64,
) -> Result<(), cairo::Error> {
    let corner = (rect.width.min(rect.height) * 0.09).clamp(12.0, 24.0);
    rounded_rect(
        context,
        rect.x + 2.0,
        rect.y + 12.0,
        rect.width,
        rect.height,
        corner,
    );
    context.set_source_rgba(0.0, 0.0, 0.0, 0.38 * progress);
    context.fill()?;

    rounded_rect(context, rect.x, rect.y, rect.width, rect.height, corner);
    context.set_source_rgba(0.075, 0.09, 0.14, 0.97);
    context.fill_preserve()?;
    context.set_source_rgba(1.0, 1.0, 1.0, 0.17);
    context.set_line_width(1.2);
    context.stroke()?;

    let icon_size = (rect.height * 0.34)
        .clamp(52.0, 92.0)
        .min(rect.width * 0.25);
    let icon_x = rect.x + 28.0;
    let icon_y = rect.y + (rect.height - icon_size) / 2.0;
    draw_app_icon(
        context,
        entry,
        icon_x,
        icon_y,
        icon_size,
        app_color(&entry.app_name),
    )?;

    if progress > 0.12 && rect.width > 160.0 {
        let text_x = icon_x + icon_size + 22.0;
        let max_text_width = (rect.x + rect.width - text_x - 24.0).max(40.0);
        context.select_font_face("Sans", FontSlant::Normal, FontWeight::Normal);
        context.set_font_size(11.0);
        draw_text(
            context,
            "SWITCHING TO",
            text_x,
            rect.y + rect.height * 0.36,
            (0.61, 0.69, 0.83, progress),
        )?;
        context.select_font_face("Sans", FontSlant::Normal, FontWeight::Bold);
        context.set_font_size(24.0_f64.min(rect.height * 0.13));
        let app_name = fit_text(context, &entry.app_name, max_text_width)?;
        draw_text(
            context,
            &app_name,
            text_x,
            rect.y + rect.height * 0.50,
            (0.97, 0.98, 1.0, progress),
        )?;
        context.select_font_face("Sans", FontSlant::Normal, FontWeight::Normal);
        context.set_font_size(13.0);
        let title = fit_text(context, &entry.title, max_text_width)?;
        draw_text(
            context,
            &title,
            text_x,
            rect.y + rect.height * 0.61,
            (0.66, 0.72, 0.81, progress),
        )?;
    }
    Ok(())
}

fn draw_app_icon(
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

fn draw_text(
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

fn fit_text(context: &Context, text: &str, max_width: f64) -> Result<String, cairo::Error> {
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

fn rounded_rect(context: &Context, x: f64, y: f64, width: f64, height: f64, radius: f64) {
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

fn app_color(name: &str) -> (f64, f64, f64) {
    let hash = name.bytes().fold(0x811c9dc5_u32, |hash, byte| {
        (hash ^ u32::from(byte)).wrapping_mul(0x01000193)
    });
    let hue = f64::from(hash % 360) / 360.0;
    let chroma = 0.48;
    let secondary = chroma * (1.0 - ((hue * 6.0) % 2.0 - 1.0).abs());
    let (red, green, blue) = match (hue * 6.0).floor() as u8 {
        0 => (chroma, secondary, 0.0),
        1 => (secondary, chroma, 0.0),
        2 => (0.0, chroma, secondary),
        3 => (0.0, secondary, chroma),
        4 => (secondary, 0.0, chroma),
        _ => (chroma, 0.0, secondary),
    };
    let offset = 0.30;
    (red + offset, green + offset, blue + offset)
}

fn lerp(from: f64, to: f64, progress: f64) -> f64 {
    from + (to - from) * progress
}

#[derive(Clone, Copy)]
struct Rect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}
