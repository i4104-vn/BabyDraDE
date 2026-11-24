//! image icon helpers.

use super::*;

pub fn get_icon_from_svg(svg_content: &str, size: i32) -> gtk4::Image {
    let bytes = glib::Bytes::from(svg_content.as_bytes());
    let stream = MemoryInputStream::from_bytes(&bytes);

    let pixbuf =
        Pixbuf::from_stream_at_scale(&stream, size, size, true, gtk4::gio::Cancellable::NONE);

    match pixbuf {
        Ok(pb) => {
            let texture = Texture::for_pixbuf(&pb);
            let img = gtk4::Image::from_paintable(Some(&texture));
            img.set_pixel_size(size);
            img
        }
        Err(_) => gtk4::Image::from_icon_name("image-missing"),
    }
}

/// Embedded bytes of the BabyDra logo.
pub const LOGO_BYTES: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/logo.png"));

/// Returns the embedded logo paintable at the requested size.
pub fn get_logo_paintable(size: i32) -> Option<Texture> {
    let bytes = glib::Bytes::from(LOGO_BYTES);
    let stream = MemoryInputStream::from_bytes(&bytes);
    Pixbuf::from_stream_at_scale(&stream, size, size, true, gtk4::gio::Cancellable::NONE)
        .ok()
        .map(|pb| Texture::for_pixbuf(&pb))
}

/// Returns the current `logo png` as a GTK Image.
pub fn get_logo_png(size: i32) -> gtk4::Image {
    if let Some(texture) = get_logo_paintable(size) {
        let img = gtk4::Image::from_paintable(Some(&texture));
        img.set_pixel_size(size);
        img
    } else {
        gtk4::Image::from_icon_name("image-missing")
    }
}

/// Returns the current `icon colored`.
pub fn get_icon_colored(name: &str, size: i32, color_hex: &str) -> gtk4::Image {
    let final_color = if !is_dark_mode() {
        let normalized = color_hex.replace(' ', "");
        match normalized.as_str() {
            "rgba(255,255,255,0.8)" | "rgba(255,255,255,0.80)" => {
                "rgba(28, 28, 30, 0.85)".to_string()
            }
            "rgba(255,255,255,0.7)" | "rgba(255,255,255,0.70)" => {
                "rgba(28, 28, 30, 0.75)".to_string()
            }
            "rgba(255,255,255,0.6)" | "rgba(255,255,255,0.60)" => {
                "rgba(28, 28, 30, 0.65)".to_string()
            }
            _ => color_hex.to_string(),
        }
    } else {
        color_hex.to_string()
    };

    if let Some((dark_svg, light_svg)) = resolve_icon_pair(name) {
        let is_dark = is_dark_mode();
        let base_svg = if is_dark { dark_svg } else { light_svg };
        let colored_svg = base_svg
            .replace("#ffffff", &final_color)
            .replace("#FFFFFF", &final_color)
            .replace("#1c1c1e", &final_color)
            .replace("#1C1C1E", &final_color);
        return get_icon_from_svg(&colored_svg, size);
    }
    get_fallback_icon(name, "image-missing")
}
