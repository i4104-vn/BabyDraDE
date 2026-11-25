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

mod png;

use cursor_icon::CursorIcon;

use super::{reader::get_cursor_theme_search_paths, xcursor::parse_xcursor, CursorImage};
use png::parse_png;

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
