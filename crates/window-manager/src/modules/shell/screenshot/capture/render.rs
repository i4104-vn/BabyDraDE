//! Screen capture and transform matrix helpers.

use std::path::PathBuf;

use cairo::{Context, Format, ImageSurface, Matrix};
use services::screenshot::{get_screenshot_path, save_png};
use services::theme::DesktopTheme;
use smithay::{
    backend::{
        allocator::Fourcc,
        egl::get_proc_address,
        renderer::{
            damage::OutputDamageTracker,
            element::RenderElement,
            gles::{ffi::Gles2, GlesRenderbuffer},
            glow::GlowRenderer,
            Bind, Offscreen,
        },
    },
    output::Output,
    utils::{Physical, Size},
};

use super::transform::transform_matrix;
use super::*;

pub(crate) fn capture_screen<E>(
    renderer: &mut GlowRenderer,
    elements: &[E],
    output: &Output,
) -> Option<PathBuf>
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
    tracing::debug!(
        "[screenshot] raw mode size {:?}, transform {:?}",
        raw,
        transform
    );
    let buffer_size = Size::<i32, smithay::utils::Buffer>::from((raw.w, raw.h));

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
    // SAFETY: Backend surface and renderer stay alive for this call.
    if let Err(e) = unsafe { renderer.egl_context().make_current() } {
        tracing::warn!("[screenshot] make_current failed: {:?}", e);
        return None;
    }
    // SAFETY: Backend surface and renderer stay alive for this call.
    let gl = Gles2::load_with(|symbol| unsafe { get_proc_address(symbol) });

    let (width, height) = (raw.w as usize, raw.h as usize);
    let mut rgba = vec![0u8; width * height * 4];
    // SAFETY: Backend surface and renderer stay alive for this call.
    unsafe {
        gl.ReadBuffer(smithay::backend::renderer::gles::ffi::COLOR_ATTACHMENT0);
        gl.ReadPixels(
            0,
            0,
            raw.w,
            raw.h,
            smithay::backend::renderer::gles::ffi::RGBA,
            smithay::backend::renderer::gles::ffi::UNSIGNED_BYTE,
            rgba.as_mut_ptr() as *mut _,
        );
    }
    tracing::debug!("[screenshot] ReadPixels done");

    // Pixel conversion, rotation and PNG encoding are CPU-heavy; keep them off
    // the compositor's event loop so taking a screenshot never stalls rendering.
    let save_path = get_screenshot_path();
    let worker_path = save_path.clone();
    let result = std::thread::Builder::new()
        .name("babydra-screenshot-save".to_string())
        .spawn(move || {
            let save_path = worker_path;
            let bgra = rgba_to_argb32(&rgba, width, height);

            let raw_surface = match ImageSurface::create_for_data(
                bgra,
                Format::ARgb32,
                width as i32,
                height as i32,
                (width * 4) as i32,
            ) {
                Ok(surface) => surface,
                Err(e) => {
                    tracing::warn!("[screenshot] cannot build source surface: {}", e);
                    return;
                }
            };

            // Rotate the physical frame into the output's logical orientation.
            let logical = transform
                .transform_size(Size::<i32, Physical>::from((width as i32, height as i32)));
            let surface = match ImageSurface::create(Format::ARgb32, logical.w, logical.h) {
                Ok(surface) => surface,
                Err(e) => {
                    tracing::warn!("[screenshot] cannot build output surface: {}", e);
                    return;
                }
            };
            {
                let Ok(cr) = Context::new(&surface) else {
                    return;
                };
                let (a, b, c, d, tx, ty) = transform_matrix(transform, width as i32, height as i32);
                cr.set_matrix(Matrix::new(a, b, c, d, tx, ty));
                if cr.set_source_surface(&raw_surface, 0.0, 0.0).is_err() {
                    return;
                }
                let _ = cr.paint();
            }

            tracing::debug!("[screenshot] saving to {}", save_path.display());
            if !save_png(&surface, &save_path) {
                return;
            }
            tracing::info!("[screenshot] saved {}", save_path.display());

            // Spawn the GTK annotation editor as soon as the image file is written
            // so the editor window appears instantly without unnecessary delays.
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
        });

    if let Err(e) = result {
        tracing::warn!("[screenshot] cannot spawn save thread: {}", e);
        return None;
    }

    Some(save_path)
}
