//! PNG cursor image decoding via cairo (no GTK involved).

use super::CursorImage;

/// Decodes PNG bytes into a [`CursorImage`], clamping the hotspot to the image.
pub(super) fn parse_png(bytes: &[u8], hotspot_x: u32, hotspot_y: u32) -> Option<CursorImage> {
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
        for chunk in row.as_chunks::<4>().0 {
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
