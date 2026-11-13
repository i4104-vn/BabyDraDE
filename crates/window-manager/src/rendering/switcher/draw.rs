use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use cairo::{Context, FontSlant, FontWeight};
use gtk4::gdk::prelude::GdkCairoContextExt;
use gtk4::gdk_pixbuf::Pixbuf;
use gtk4::gio::prelude::*;

use crate::compositor::AltTabEntry;

thread_local! {
    static ICON_CACHE: RefCell<HashMap<(String, i32), Option<Pixbuf>>> = RefCell::new(HashMap::new());
}

pub(crate) fn resolve_app_icon(app_id: &str, app_name: &str, size: i32) -> Option<Pixbuf> {
    let key = (app_id.to_string(), size);
    ICON_CACHE.with(|cache| {
        let mut map = cache.borrow_mut();
        if let Some(cached) = map.get(&key) {
            return cached.clone();
        }
        let pixbuf = load_app_icon_pixbuf(app_id, app_name, size);
        map.insert(key, pixbuf.clone());
        pixbuf
    })
}

fn load_app_icon_pixbuf(app_id: &str, app_name: &str, size: i32) -> Option<Pixbuf> {
    let clean_id = app_id
        .trim()
        .strip_suffix(".desktop")
        .unwrap_or(app_id.trim());

    let mut candidates = Vec::new();

    // 1. Look up via gio::AppInfo (matches against system .desktop files)
    if !clean_id.is_empty() {
        for info in gtk4::gio::AppInfo::all() {
            if let Some(id) = info.id() {
                let id_clean = id.strip_suffix(".desktop").unwrap_or(id.as_str());
                if id_clean.eq_ignore_ascii_case(clean_id) {
                    if let Some(icon) = info.icon() {
                        if let Ok(themed) = icon.clone().downcast::<gtk4::gio::ThemedIcon>() {
                            for name in themed.names() {
                                candidates.push(name.to_string());
                            }
                        }
                        if let Ok(file_icon) = icon.clone().downcast::<gtk4::gio::FileIcon>() {
                            if let Some(path) = file_icon.file().path() {
                                candidates.push(path.to_string_lossy().to_string());
                            }
                        }
                        if let Some(to_str) = icon.to_string() {
                            if !to_str.is_empty() {
                                candidates.push(to_str.to_string());
                            }
                        }
                    }
                    break;
                }
            }
        }
    }

    // 2. Direct app_id and name candidates
    if !clean_id.is_empty() {
        candidates.push(clean_id.to_string());
        candidates.push(clean_id.to_lowercase());
        if let Some(last) = clean_id.split('.').next_back() {
            candidates.push(last.to_string());
            candidates.push(last.to_lowercase());
        }
    }
    let clean_name = app_name.trim();
    if !clean_name.is_empty() {
        candidates.push(clean_name.to_lowercase());
    }

    // 3. Direct absolute paths (e.g. Icon=/usr/share/babydra/logo.png)
    for c in &candidates {
        if c.starts_with('/') {
            let p = Path::new(c);
            if p.is_file() {
                if let Ok(pb) = Pixbuf::from_file_at_scale(p, size, size, true) {
                    return Some(pb);
                }
            }
        }
    }

    // 4. Look up in active system icon theme
    let theme = if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::IconTheme::for_display(&display)
    } else {
        let t = gtk4::IconTheme::new();
        let theme_name = services::theme::gtk::read_gtk_settings().icon_theme;
        if !theme_name.is_empty() {
            t.set_theme_name(Some(&theme_name));
        }
        t
    };

    for c in &candidates {
        if c.starts_with('/') {
            continue;
        }
        let clean = c
            .strip_suffix(".png")
            .or_else(|| c.strip_suffix(".svg"))
            .unwrap_or(c);

        let paintable = theme.lookup_icon(
            clean,
            &[],
            size,
            1,
            gtk4::TextDirection::None,
            gtk4::IconLookupFlags::empty(),
        );
        if let Some(file) = paintable.file() {
            if let Some(path) = file.path() {
                if let Ok(pb) = Pixbuf::from_file_at_scale(&path, size, size, true) {
                    return Some(pb);
                }
            }
        }
    }

    // 5. Look up in /usr/share/pixmaps
    for c in &candidates {
        let clean = c
            .strip_suffix(".png")
            .or_else(|| c.strip_suffix(".svg"))
            .unwrap_or(c);
        for ext in &[".png", ".svg"] {
            let p = PathBuf::from(format!("/usr/share/pixmaps/{}{}", clean, ext));
            if p.is_file() {
                if let Ok(pb) = Pixbuf::from_file_at_scale(&p, size, size, true) {
                    return Some(pb);
                }
            }
        }
    }

    // 6. Theme generic fallbacks
    for fallback in &["application-x-executable", "image-missing"] {
        let paintable = theme.lookup_icon(
            fallback,
            &[],
            size,
            1,
            gtk4::TextDirection::None,
            gtk4::IconLookupFlags::empty(),
        );
        if let Some(file) = paintable.file() {
            if let Some(path) = file.path() {
                if let Ok(pb) = Pixbuf::from_file_at_scale(&path, size, size, true) {
                    return Some(pb);
                }
            }
        }
    }

    None
}

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

    let target_size = size.round().max(16.0) as i32;
    if let Some(pixbuf) = resolve_app_icon(&entry.app_id, &entry.app_name, target_size) {
        context.save()?;
        // Soft drop shadow behind icon
        let corner = size * 0.22;
        rounded_rect(context, x, y + 1.5, size, size, corner);
        context.set_source_rgba(0.0, 0.0, 0.0, 0.35 * alpha);
        context.fill()?;

        let pw = pixbuf.width() as f64;
        let ph = pixbuf.height() as f64;
        if pw > 0.0 && ph > 0.0 {
            let sx = size / pw;
            let sy = size / ph;
            context.translate(x, y);
            context.scale(sx, sy);
            context.set_source_pixbuf(&pixbuf, 0.0, 0.0);
            context.paint_with_alpha(alpha)?;
        }
        context.restore()?;
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
