//! Direct compositor screenshot capture — no external capture utilities.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use cairo::{Context, Format, ImageSurface, Matrix};
use services::theme::DesktopTheme;
use smithay::{
    backend::{
        allocator::Fourcc,
        egl::get_proc_address,
        renderer::{
            damage::OutputDamageTracker,
            element::RenderElement,
            gles::{ffi as gl_ffi, ffi::Gles2, GlesRenderbuffer},
            glow::GlowRenderer,
            Bind, Offscreen,
        },
    },
    output::Output,
    utils::{Buffer as BufferCoord, Size, Transform},
};

/// Shared flag set by the keymap thread and consumed by the render loops.
static CAPTURE_REQUESTED: AtomicBool = AtomicBool::new(false);

/// Resolves the default file path to save screenshots in `~/Pictures/Screenshots/`.
pub fn get_screenshot_path() -> PathBuf {
    let pictures_dir = dirs::picture_dir().unwrap_or_else(|| {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        home.join("Pictures")
    });
    let screenshots_dir = pictures_dir.join("Screenshots");
    let _ = std::fs::create_dir_all(&screenshots_dir);

    let datetime = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    screenshots_dir.join(format!("Screenshot_{}.png", datetime))
}

/// Requests a screenshot. Safe to call from any thread.
pub fn trigger() {
    CAPTURE_REQUESTED.store(true, Ordering::SeqCst);
}

/// Consumes a pending screenshot request. Called once per rendered frame.
pub fn take_requested() -> bool {
    CAPTURE_REQUESTED.swap(false, Ordering::SeqCst)
}

/// Captures the desktop frame directly through the compositor renderer and
/// saves it as a lossless PNG. Opens the annotation editor on the saved image.
pub fn capture_screen<E>(renderer: &mut GlowRenderer, elements: &[E], output: &Output) -> Option<PathBuf>
where
    E: RenderElement<GlowRenderer>,
{
    tracing::info!("[screenshot] capturing screen");

    let mode = output.current_mode()?;
    let transform = output.current_transform();

    // OutputDamageTracker renders into the *raw* mode size and applies the
    // inverse output transform itself. The offscreen target must use that same
    // size, otherwise a rotated output yields a sideways, partially blank PNG.
    let raw = mode.size;
    tracing::debug!("[screenshot] raw mode size {:?}, transform {:?}", raw, transform);
    let buffer_size = Size::<i32, BufferCoord>::from((raw.w, raw.h));

    let mut target: GlesRenderbuffer = match renderer.create_buffer(Fourcc::Argb8888, buffer_size) {
        Ok(target) => target,
        Err(e) => {
            tracing::warn!("[screenshot] create_buffer failed: {}", e);
            return None;
        }
    };
    let mut framebuffer = match renderer.bind(&mut target) {
        Ok(fb) => fb,
        Err(e) => {
            tracing::warn!("[screenshot] bind offscreen target failed: {}", e);
            return None;
        }
    };
    let clear_color = DesktopTheme::default().background_color;
    if let Err(e) = OutputDamageTracker::from_output(output).render_output(
        renderer,
        &mut framebuffer,
        0,
        elements,
        clear_color,
    ) {
        tracing::warn!("[screenshot] render_output failed: {}", e);
        return None;
    }

    // get_proc_address() is only safe while an EGL context is current.
    if let Err(e) = unsafe { renderer.egl_context().make_current() } {
        tracing::warn!("[screenshot] make_current failed: {:?}", e);
        return None;
    }
    let gl = Gles2::load_with(|symbol| unsafe { get_proc_address(symbol) });

    let (width, height) = (raw.w as usize, raw.h as usize);
    let mut rgba = vec![0u8; width * height * 4];
    unsafe {
        gl.ReadBuffer(gl_ffi::COLOR_ATTACHMENT0);
        gl.ReadPixels(
            0,
            0,
            raw.w,
            raw.h,
            gl_ffi::RGBA,
            gl_ffi::UNSIGNED_BYTE,
            rgba.as_mut_ptr() as *mut _,
        );
    }
    tracing::debug!("[screenshot] ReadPixels done");

    // GL rows are bottom-up and RGBA; cairo wants top-down premultiplied ARGB32
    // (BGRA byte order on little-endian).
    let mut bgra = vec![0u8; rgba.len()];
    let row = width * 4;
    for y in 0..height {
        let src = &rgba[(height - 1 - y) * row..(height - y) * row];
        let dst = &mut bgra[y * row..(y + 1) * row];
        for x in 0..width {
            let (s, d) = (&src[x * 4..x * 4 + 4], &mut dst[x * 4..x * 4 + 4]);
            d[0] = s[2];
            d[1] = s[1];
            d[2] = s[0];
            d[3] = s[3];
        }
    }

    let raw_surface = ImageSurface::create_for_data(bgra, Format::ARgb32, raw.w, raw.h, row as i32)
        .ok()?;

    // Rotate the physical frame into the output's logical orientation so the
    // saved image matches what is shown on screen (also handles flipped modes).
    let logical = transform.transform_size(raw);
    let surface = ImageSurface::create(Format::ARgb32, logical.w, logical.h).ok()?;
    {
        let cr = Context::new(&surface).ok()?;
        let (a, b, c, d, tx, ty) = transform_matrix(rotation_only(transform), raw.w, raw.h);
        cr.set_matrix(Matrix::new(a, b, c, d, tx, ty));
        cr.set_source_surface(&raw_surface, 0.0, 0.0).ok()?;
        cr.paint().ok()?;
    }

    let save_path = get_screenshot_path();
    tracing::debug!("[screenshot] saving to {}", save_path.display());
    {
        let mut file = std::fs::File::create(&save_path).ok()?;
        surface.write_to_png(&mut file).ok()?;
    }
    tracing::info!("[screenshot] saved {}", save_path.display());

    let notif_title = services::trans("screenshot.full_saved_title");
    let notif_msg =
        services::trans("screenshot.saved_msg").replace("{}", &format!("{:?}", save_path));
    send_notification(&notif_title, &notif_msg);

    // Run the GTK annotation editor in a separate process. GTK cannot run on a
    // worker thread of this process: the compositor already owns the default
    // GLib main context, which makes g_application_run() abort the whole WM.
    let editor_path = save_path.to_string_lossy().into_owned();
    match std::env::current_exe() {
        Ok(exe) => {
            if let Err(e) = std::process::Command::new(exe)
                .arg("--screenshot-editor")
                .arg(&editor_path)
                .spawn()
            {
                tracing::warn!("[screenshot] cannot start annotation editor: {}", e);
            }
        }
        Err(e) => tracing::warn!("[screenshot] cannot locate compositor binary: {}", e),
    }

    Some(save_path)
}

/// Keeps only the rotational part of an output transform.
fn rotation_only(transform: Transform) -> Transform {
    match transform {
        Transform::Flipped => Transform::Normal,
        Transform::Flipped90 => Transform::_90,
        Transform::Flipped180 => Transform::_180,
        Transform::Flipped270 => Transform::_270,
        other => other,
    }
}

/// Affine matrix mapping the raw frame buffer into the output's logical
/// orientation. Coefficients come from smithay's `Transform::transform_point_in`.
fn transform_matrix(transform: Transform, w: i32, h: i32) -> (f64, f64, f64, f64, f64, f64) {
    let (w, h) = (w as f64, h as f64);
    match transform {
        Transform::Normal => (1.0, 0.0, 0.0, 1.0, 0.0, 0.0),
        Transform::_90 => (0.0, 1.0, -1.0, 0.0, h, 0.0),
        Transform::_180 => (-1.0, 0.0, 0.0, -1.0, w, h),
        Transform::_270 => (0.0, -1.0, 1.0, 0.0, 0.0, w),
        Transform::Flipped => (-1.0, 0.0, 0.0, 1.0, w, 0.0),
        Transform::Flipped90 => (0.0, 1.0, 1.0, 0.0, 0.0, 0.0),
        Transform::Flipped180 => (1.0, 0.0, 0.0, -1.0, 0.0, h),
        Transform::Flipped270 => (0.0, -1.0, -1.0, 0.0, h, w),
    }
}

/// Dispatches a desktop notification.
pub fn send_notification(title: &str, body: &str) {
    let _ = std::process::Command::new("notify-send")
        .args(["-a", "BabyDra", title, body])
        .spawn();
}

#[cfg(test)]
mod tests {
    use super::*;
    use smithay::utils::{Physical, Point};

    #[test]
    fn transform_matrix_matches_smithay_point_mapping() {
        let (w, h) = (13, 7);
        let area = Size::<i32, Physical>::from((w, h));
        let transforms = [
            Transform::Normal,
            Transform::_90,
            Transform::_180,
            Transform::_270,
            Transform::Flipped,
            Transform::Flipped90,
            Transform::Flipped180,
            Transform::Flipped270,
        ];

        for transform in transforms {
            let (a, b, c, d, tx, ty) = transform_matrix(transform, w, h);
            for x in 0..=w {
                for y in 0..=h {
                    let expected = transform.transform_point_in(
                        Point::<i32, Physical>::from((x, y)),
                        &area,
                    );
                    let actual = (
                        (a * x as f64 + c * y as f64 + tx) as i32,
                        (b * x as f64 + d * y as f64 + ty) as i32,
                    );
                    assert_eq!(actual, (expected.x, expected.y), "transform {:?}", transform);
                }
            }
        }
    }
}
