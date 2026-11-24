//! theme icon helpers.

use super::*;

/// Retrieves a Pixbuf for an icon name or path, prioritizing system GtkIconTheme.
///
/// Returns `None` (never panics) when GTK is not initialized or there is no
/// default GDK display, so callers such as the Alt+Tab switcher can degrade
/// gracefully to a coloured badge instead of crashing the compositor.
pub fn get_theme_icon_pixbuf(name: &str, size: i32) -> Option<Pixbuf> {
    let path = std::path::Path::new(name);
    if path.is_absolute() && path.is_file() {
        return Pixbuf::from_file_at_scale(path, size, size, true).ok();
    }

    // Without a default GDK display there is no icon theme to query.  Do not
    // fall back to `IconTheme::new()` — a theme created without a display can
    // make `has_icon`/`lookup_icon` crash on some GTK builds.
    let disp = match gtk4::gdk::Display::default() {
        Some(d) => d,
        None => {
            tracing::warn!(
                "get_theme_icon_pixbuf: no default GDK display; icon '{}' unavailable",
                name
            );
            return None;
        }
    };
    let theme = gtk4::IconTheme::for_display(&disp);

    let clean = name
        .strip_suffix(".png")
        .or_else(|| name.strip_suffix(".svg"))
        .unwrap_or(name);

    if theme.has_icon(clean) {
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

    // Fallback: check pixmaps
    for ext in &[".png", ".svg"] {
        let p = std::path::PathBuf::from(format!("/usr/share/pixmaps/{}{}", clean, ext));
        if p.is_file() {
            if let Ok(pb) = Pixbuf::from_file_at_scale(&p, size, size, true) {
                return Some(pb);
            }
        }
    }

    // Fallback: resolve local SVG in ui-kit
    if let Some((dark_svg, light_svg)) = resolve_icon_pair(clean) {
        let is_dark = is_dark_mode();
        let svg = if is_dark { dark_svg } else { light_svg };
        let bytes = glib::Bytes::from(svg.as_bytes());
        let stream = MemoryInputStream::from_bytes(&bytes);
        return Pixbuf::from_stream_at_scale(
            &stream,
            size,
            size,
            true,
            gtk4::gio::Cancellable::NONE,
        )
        .ok();
    }

    // Generic fallback icon
    for fallback in &["application-x-executable", "image-missing"] {
        if theme.has_icon(fallback) {
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
    }

    None
}
