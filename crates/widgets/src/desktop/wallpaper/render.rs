use gtk4::cairo;
use gtk4::gdk::prelude::GdkCairoContextExt;
use gtk4::gdk_pixbuf::Pixbuf;
use gtk4::gio::MemoryInputStream;
use gtk4::glib;
use services::wallpaper::get_default_wallpaper_bytes;
use std::cell::RefCell;
use std::path::{Path, PathBuf};

type CachedWallpaperEntry = (Option<PathBuf>, (i32, i32), Pixbuf);

thread_local! {
    static CACHED_WALLPAPER: RefCell<Option<CachedWallpaperEntry>> = const { RefCell::new(None) };
}

pub fn render_wallpaper_background(
    cr: &cairo::Context,
    width: f64,
    height: f64,
    wallpaper_path: Option<&Path>,
) {
    let target_w = width.round() as i32;
    let target_h = height.round() as i32;
    if target_w <= 0 || target_h <= 0 {
        return;
    }

    let Some(pixbuf) = wallpaper_pixbuf(wallpaper_path, target_w, target_h) else {
        // Default Fluent deep dark background fallback
        let gradient = cairo::LinearGradient::new(0.0, 0.0, width, height);
        gradient.add_color_stop_rgb(0.0, 0.08, 0.09, 0.12);
        gradient.add_color_stop_rgb(1.0, 0.04, 0.05, 0.07);
        cr.set_source(&gradient).ok();
        let _ = cr.paint();
        return;
    };

    let img_w = pixbuf.width() as f64;
    let img_h = pixbuf.height() as f64;
    let offset_x = (width - img_w) / 2.0;
    let offset_y = (height - img_h) / 2.0;

    let _ = cr.save();
    cr.set_source_pixbuf(&pixbuf, offset_x, offset_y);
    let _ = cr.paint();
    let _ = cr.restore();
}

/// Decodes/scales the image once before the reveal begins. Subsequent frames
/// only paint the cached pixbuf through a changing clip.
pub fn prepare_wallpaper_background(width: f64, height: f64, wallpaper_path: Option<&Path>) {
    let target_w = width.round() as i32;
    let target_h = height.round() as i32;
    if target_w > 0 && target_h > 0 {
        let _ = wallpaper_pixbuf(wallpaper_path, target_w, target_h);
    }
}

fn wallpaper_pixbuf(wallpaper_path: Option<&Path>, target_w: i32, target_h: i32) -> Option<Pixbuf> {
    let current_path = wallpaper_path.map(Path::to_path_buf);
    let cached = CACHED_WALLPAPER.with(|c| {
        if let Some((ref p, (w, h), ref pixbuf)) = *c.borrow() {
            if *p == current_path && w == target_w && h == target_h {
                return Some(pixbuf.clone());
            }
        }
        None
    });

    if let Some(pb) = cached {
        Some(pb)
    } else {
        let loaded = load_wallpaper_pixbuf(current_path.as_deref(), target_w, target_h);
        if let Some(ref pb) = loaded {
            CACHED_WALLPAPER.with(|c| {
                *c.borrow_mut() = Some((current_path, (target_w, target_h), pb.clone()));
            });
        }
        loaded
    }
}

/// Paints the wallpaper through a growing circular reveal. The image itself
/// remains cached by `render_wallpaper_background`, so animation frames only
/// change the Cairo clip and never reload the bitmap.
pub fn render_wallpaper_reveal(
    cr: &cairo::Context,
    width: f64,
    height: f64,
    wallpaper_path: Option<&Path>,
    center: (f64, f64),
    progress: f64,
) {
    cr.set_source_rgb(0.02, 0.025, 0.04);
    let _ = cr.paint();
    let max_radius = [center.0, width - center.0, center.1, height - center.1]
        .into_iter()
        .map(|v| v.abs())
        .fold(0.0_f64, f64::max)
        * 1.45;
    let _ = cr.save();
    cr.arc(
        center.0,
        center.1,
        (max_radius * progress).max(1.0),
        0.0,
        std::f64::consts::TAU,
    );
    cr.clip();
    render_wallpaper_background(cr, width, height, wallpaper_path);
    let _ = cr.restore();
}

fn load_wallpaper_pixbuf(path: Option<&Path>, target_w: i32, target_h: i32) -> Option<Pixbuf> {
    // 1. Try loading specified wallpaper from file
    if let Some(p) = path {
        if p.exists() {
            let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("");
            if ext.eq_ignore_ascii_case("svg") {
                if let Ok(pb) = Pixbuf::from_file_at_scale(p, target_w, target_h, false) {
                    return Some(pb);
                }
            } else if let Ok(pb) = Pixbuf::from_file(p) {
                return scale_aspect_fill(&pb, target_w, target_h);
            }
        }
    }

    // 2. Fallback to embedded default wallpaper (wallpaper.jpeg)
    let wallpaper_bytes = get_default_wallpaper_bytes();
    let bytes = glib::Bytes::from(wallpaper_bytes);
    let stream = MemoryInputStream::from_bytes(&bytes);
    if let Ok(pb) = Pixbuf::from_stream(&stream, gtk4::gio::Cancellable::NONE) {
        return scale_aspect_fill(&pb, target_w, target_h);
    }

    None
}

fn scale_aspect_fill(src: &Pixbuf, target_w: i32, target_h: i32) -> Option<Pixbuf> {
    let src_w = src.width() as f64;
    let src_h = src.height() as f64;
    if src_w <= 0.0 || src_h <= 0.0 {
        return None;
    }

    let scale_x = target_w as f64 / src_w;
    let scale_y = target_h as f64 / src_h;
    let scale = scale_x.max(scale_y);

    let new_w = (src_w * scale).round() as i32;
    let new_h = (src_h * scale).round() as i32;

    src.scale_simple(new_w, new_h, gtk4::gdk_pixbuf::InterpType::Bilinear)
}
