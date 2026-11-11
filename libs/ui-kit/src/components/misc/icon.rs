use gtk4::prelude::*;
use gtk4::{Image, Label, Widget};
use std::path::Path;

/// Helper to construct icons from SVG files or Unicode font glyphs
pub struct Icon;

impl Icon {
    pub fn from_file_or_glyph(file_path: &str, glyph_fallback: &str, pixel_size: i32) -> Widget {
        if Path::new(file_path).exists() {
            let img = Image::from_file(file_path);
            img.set_pixel_size(pixel_size);
            img.upcast()
        } else {
            let label = Label::new(Some(glyph_fallback));
            label.upcast()
        }
    }
}
