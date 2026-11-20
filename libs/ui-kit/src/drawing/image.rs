//! Image processing helpers: square cropping and circular alpha masking.

/// Converts raw image bytes into a square, centered, scaled `Pixbuf`.
pub fn crop_square(bytes: &[u8], size: i32) -> Option<gdk_pixbuf::Pixbuf> {
    let stream = gtk4::gio::MemoryInputStream::from_bytes(&gtk4::glib::Bytes::from(bytes));
    let pixbuf = gdk_pixbuf::Pixbuf::from_stream(&stream, gtk4::gio::Cancellable::NONE).ok()?;
    crop_square_pixbuf(&pixbuf, size)
}

/// Center-crops an existing `Pixbuf` to a 1:1 square and scales it to `size x size`.
pub fn crop_square_pixbuf(pixbuf: &gdk_pixbuf::Pixbuf, size: i32) -> Option<gdk_pixbuf::Pixbuf> {
    let w = pixbuf.width();
    let h = pixbuf.height();
    if w <= 0 || h <= 0 {
        return None;
    }
    let min_dim = std::cmp::min(w, h);
    let x = (w - min_dim) / 2;
    let y = (h - min_dim) / 2;

    let sub = pixbuf.new_subpixbuf(x, y, min_dim, min_dim);
    sub.scale_simple(size, size, gdk_pixbuf::InterpType::Bilinear)
}

/// Applies an anti-aliased circular alpha mask to a square pixbuf.
pub fn apply_circular_mask(pixbuf: &gdk_pixbuf::Pixbuf) -> gdk_pixbuf::Pixbuf {
    let w = pixbuf.width();
    let h = pixbuf.height();
    let n_channels = pixbuf.n_channels();
    let rowstride = pixbuf.rowstride();

    let src_bytes = pixbuf.pixel_bytes();
    let src = src_bytes.as_ref().map(|b| b.as_ref()).unwrap_or(&[]);

    let Some(out) = gdk_pixbuf::Pixbuf::new(gdk_pixbuf::Colorspace::Rgb, true, 8, w, h) else {
        return pixbuf.clone();
    };

    let out_stride = out.rowstride() as usize;
    let out_n_channels = out.n_channels() as usize;
    let out_pixels = unsafe { out.pixels() };

    let center_x = (w - 1) as f64 / 2.0;
    let center_y = (h - 1) as f64 / 2.0;
    let radius = (w.min(h) as f64 - 1.0) / 2.0;
    let feather = 1.5 / radius.max(1.0);

    let src_stride = rowstride as usize;
    let src_channels = n_channels as usize;

    for y in 0..h {
        let y_f = y as f64;
        let dy = (y_f - center_y) / radius.max(1.0);
        let dy2 = dy * dy;
        let src_row_pos = (y as usize) * src_stride;
        let out_row_pos = (y as usize) * out_stride;

        for x in 0..w {
            let x_f = x as f64;
            let dx = (x_f - center_x) / radius.max(1.0);
            let dist = (dx * dx + dy2).sqrt();
            let alpha = (((1.0 - dist) / feather).clamp(0.0, 1.0) * 255.0).round() as u8;

            let pos = src_row_pos + (x as usize) * src_channels;
            let (r, g, b) = match src.get(pos..pos + 3) {
                Some(rgb) => (rgb[0], rgb[1], rgb[2]),
                None => (0, 0, 0),
            };

            let out_pos = out_row_pos + (x as usize) * out_n_channels;
            if out_pos + 4 <= out_pixels.len() {
                out_pixels[out_pos] = r;
                out_pixels[out_pos + 1] = g;
                out_pixels[out_pos + 2] = b;
                out_pixels[out_pos + 3] = alpha;
            }
        }
    }
    out
}

/// Converts raw image bytes into a square, scaled, circularly masked `Pixbuf`.
pub fn crop_circle(bytes: &[u8], size: i32) -> Option<gdk_pixbuf::Pixbuf> {
    let square = crop_square(bytes, size)?;
    Some(apply_circular_mask(&square))
}
