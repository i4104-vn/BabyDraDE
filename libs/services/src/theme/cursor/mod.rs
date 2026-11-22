pub mod loader;
pub mod reader;
pub mod xcursor;

pub use cursor_icon::CursorIcon;
pub use loader::{load_cursor_icon, load_cursor_image};
pub use reader::{
    find_cursor_theme_path, list_available_cursor_themes, read_active_cursor_theme,
    read_default_index_theme, CursorThemeInfo,
};

/// A decoded cursor image ready to be uploaded into a compositor memory buffer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CursorImage {
    pub width: u32,
    pub height: u32,
    /// Horizontal hotspot in pixels, measured from the image's top-left corner.
    pub xhot: u32,
    /// Vertical hotspot in pixels, measured from the image's top-left corner.
    pub yhot: u32,
    /// Premultiplied ARGB pixels in `B, G, R, A` byte order, matching the
    /// memory layout Smithay expects for `Fourcc::Argb8888` buffers.
    pub pixels: Vec<u8>,
}

impl CursorImage {
    /// Builds an image from straight-alpha `R, G, B, A` bytes, premultiplying
    /// the colour channels so the pixels can be blended directly by the renderer.
    pub fn from_rgba(width: u32, height: u32, xhot: u32, yhot: u32, rgba: &[u8]) -> Option<Self> {
        if width == 0 || height == 0 {
            return None;
        }
        let expected = width as usize * height as usize * 4;
        if rgba.len() != expected {
            return None;
        }

        let mut pixels = Vec::with_capacity(expected);
        for chunk in rgba.as_chunks::<4>().0 {
            let (r, g, b, a) = (chunk[0], chunk[1], chunk[2], chunk[3]);
            pixels.extend_from_slice(&[premultiply(b, a), premultiply(g, a), premultiply(r, a), a]);
        }

        Some(Self {
            width,
            height,
            xhot,
            yhot,
            pixels,
        })
    }
}

#[inline]
fn premultiply(channel: u8, alpha: u8) -> u8 {
    ((channel as u16 * alpha as u16 + 127) / 255) as u8
}
