//! Resolves configured cursor themes into decoded images.
//!
//! Themes follow the GTK/Xcursor layout:
//!
//! ```text
//! ~/.babydra/cursor/<theme>/
//!   index.theme        # optional, [Icon Theme] with Inherits=
//!   cursors/
//!     left_ptr         # Xcursor image (no extension)
//!     text.png         # PNG image (alternative)
//! ```

use std::fs;

use cursor_icon::CursorIcon;

use super::{reader::get_cursor_theme_search_paths, xcursor::parse_xcursor, CursorImage};

const MAX_INHERIT_DEPTH: usize = 8;

/// Loads the default pointer image for `theme`.
///
/// Kept as a convenience wrapper around [`load_cursor_icon`].
pub fn load_cursor_image(
    theme: &str,
    size: u32,
    hotspot_x: u32,
    hotspot_y: u32,
) -> Option<CursorImage> {
    load_cursor_icon(theme, CursorIcon::Default, size, hotspot_x, hotspot_y)
}

/// Loads the image for a specific cursor role (arrow, text, wait, resize, ...).
///
/// The W3C name of the icon is tried first, followed by its legacy Xcursor
/// aliases, so themes that use either naming convention work. Theme
/// inheritance from `index.theme` is honoured when an image is missing.
pub fn load_cursor_icon(
    theme: &str,
    icon: CursorIcon,
    size: u32,
    hotspot_x: u32,
    hotspot_y: u32,
) -> Option<CursorImage> {
    let theme = theme.trim();
    if theme.is_empty() {
        return None;
    }

    let mut names: Vec<&str> = vec![icon.name()];
    names.extend_from_slice(icon.alt_names());

    let mut visited = Vec::new();
    let mut queue = vec![theme.to_string()];

    while let Some(name) = queue.pop() {
        if visited.iter().any(|seen| seen == &name) {
            continue;
        }
        visited.push(name.clone());

        if let Some(image) = load_from_theme(&name, &names, size, hotspot_x, hotspot_y) {
            return Some(image);
        }

        if visited.len() >= MAX_INHERIT_DEPTH {
            break;
        }
        for parent in read_inherits(&name) {
            queue.push(parent);
        }
    }

    None
}

fn load_from_theme(
    theme: &str,
    names: &[&str],
    size: u32,
    hotspot_x: u32,
    hotspot_y: u32,
) -> Option<CursorImage> {
    for dir in get_cursor_theme_search_paths()
        .into_iter()
        .map(|base| base.join(theme).join("cursors"))
        .filter(|path| path.is_dir())
    {
        for name in names {
            let xcursor_path = dir.join(name);
            if let Ok(bytes) = fs::read(&xcursor_path) {
                if let Some(image) = parse_xcursor(&bytes, size) {
                    return Some(image);
                }
            }

            let png_path = dir.join(format!("{name}.png"));
            if let Ok(bytes) = fs::read(&png_path) {
                if let Some(image) = parse_png(&bytes, hotspot_x, hotspot_y) {
                    return Some(image);
                }
            }
        }
    }

    None
}

/// Reads `Inherits=` from a theme's `index.theme` `[Icon Theme]` section.
fn read_inherits(theme: &str) -> Vec<String> {
    for base in get_cursor_theme_search_paths() {
        let path = base.join(theme).join("index.theme");
        let Ok(content) = fs::read_to_string(path) else {
            continue;
        };

        let mut in_icon_theme = false;
        let mut parents = Vec::new();
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('[') {
                in_icon_theme = line.eq_ignore_ascii_case("[Icon Theme]");
                continue;
            }
            if !in_icon_theme {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            if key.trim().eq_ignore_ascii_case("Inherits") {
                parents.extend(
                    value
                        .split(',')
                        .map(str::trim)
                        // Some themes quote values, e.g. Inherits="hicolor".
                        .map(|parent| parent.trim_matches('"').trim_matches('\''))
                        .filter(|parent| !parent.is_empty())
                        .map(str::to_string),
                );
            }
        }
        return parents;
    }

    Vec::new()
}

/// Decodes a PNG cursor image with cairo (no GTK involved).
fn parse_png(bytes: &[u8], hotspot_x: u32, hotspot_y: u32) -> Option<CursorImage> {
    let surface = cairo::ImageSurface::create_from_png(&mut &bytes[..]).ok()?;
    let width = surface.width() as u32;
    let height = surface.height() as u32;
    if width == 0 || height == 0 || width > 512 || height > 512 {
        return None;
    }

    // Cairo PNGs with alpha load as ARGB32; opaque ones may use another format.
    // Convert those so the pixel extraction below has a single layout to handle.
    let mut surface = if surface.format() == cairo::Format::ARgb32 {
        surface
    } else {
        let converted =
            cairo::ImageSurface::create(cairo::Format::ARgb32, width as i32, height as i32).ok()?;
        {
            let context = cairo::Context::new(&converted).ok()?;
            context.set_source_surface(&surface, 0.0, 0.0).ok()?;
            context.paint().ok()?;
        }
        converted
    };

    let stride = surface.stride().max(0) as usize;
    let row_bytes = width as usize * 4;
    let data = surface.data().ok()?;
    if stride < row_bytes || data.len() < stride * height as usize {
        return None;
    }

    let mut pixels = Vec::with_capacity(row_bytes * height as usize);
    for y in 0..height as usize {
        let row = &data[y * stride..y * stride + row_bytes];
        for chunk in row.chunks_exact(4) {
            // cairo ARGB32 is premultiplied and stored in native byte order.
            let argb = u32::from_ne_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            let a = (argb >> 24) as u8;
            let r = ((argb >> 16) & 0xff) as u8;
            let g = ((argb >> 8) & 0xff) as u8;
            let b = (argb & 0xff) as u8;
            pixels.extend_from_slice(&[b, g, r, a]);
        }
    }

    let xhot = hotspot_x.min(width.saturating_sub(1));
    let yhot = hotspot_y.min(height.saturating_sub(1));

    Some(CursorImage {
        width,
        height,
        xhot,
        yhot,
        pixels,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_theme_returns_none() {
        assert!(load_cursor_image("", 24, 0, 0).is_none());
        assert!(load_cursor_image("__babydra_no_such_theme__", 24, 0, 0).is_none());
        assert!(
            load_cursor_icon("__babydra_no_such_theme__", CursorIcon::Text, 24, 0, 0).is_none()
        );
    }

    #[test]
    fn icon_names_cover_standard_and_legacy_names() {
        assert_eq!(CursorIcon::Text.name(), "text");
        assert!(CursorIcon::Text.alt_names().contains(&"ibeam"));
        assert_eq!(CursorIcon::NwseResize.name(), "nwse-resize");
        assert!(CursorIcon::Wait.alt_names().contains(&"watch"));
    }
}
