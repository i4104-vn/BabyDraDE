use gtk4::cairo;
use gtk4::gdk::prelude::GdkCairoContextExt;
use gtk4::gdk_pixbuf::Pixbuf;
use std::path::Path;

pub fn render_wallpaper_background(
    cr: &cairo::Context,
    width: f64,
    height: f64,
    wallpaper_path: Option<&Path>,
) {
    if let Some(path) = wallpaper_path {
        if path.exists() {
            if let Ok(pixbuf) = Pixbuf::from_file(path) {
                let img_w = pixbuf.width() as f64;
                let img_h = pixbuf.height() as f64;
                if img_w > 0.0 && img_h > 0.0 {
                    let scale_x = width / img_w;
                    let scale_y = height / img_h;
                    let scale = scale_x.max(scale_y);

                    let scaled_w = img_w * scale;
                    let scaled_h = img_h * scale;
                    let offset_x = (width - scaled_w) / 2.0;
                    let offset_y = (height - scaled_h) / 2.0;

                    let _ = cr.save();
                    cr.translate(offset_x, offset_y);
                    cr.scale(scale, scale);
                    cr.set_source_pixbuf(&pixbuf, 0.0, 0.0);
                    let _ = cr.paint();
                    let _ = cr.restore();
                    return;
                }
            }
        }
    }

    // Default Fluent deep dark background
    let gradient = cairo::LinearGradient::new(0.0, 0.0, width, height);
    gradient.add_color_stop_rgb(0.0, 0.08, 0.09, 0.12);
    gradient.add_color_stop_rgb(1.0, 0.04, 0.05, 0.07);
    cr.set_source(&gradient).ok();
    let _ = cr.paint();
}
