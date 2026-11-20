//! Direct compositor screenshot capture — no external capture utilities.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use cairo::{Format, ImageSurface};
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
    utils::{Buffer as BufferCoord, Size},
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

/// Requests a screenshot. Safe to call from any thread (the keymap daemon).
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
    tracing::debug!("[screenshot] mode size: {:?}", mode.size);
    let size = output.current_transform().transform_size(mode.size);
    tracing::debug!("[screenshot] transformed size: {:?}", size);
    if size.w <= 0 || size.h <= 0 {
        tracing::warn!("[screenshot] invalid size");
        return None;
    }
    let buffer_size = Size::<i32, BufferCoord>::from((size.w, size.h));

    // Render the exact same element list the backend draws into an offscreen target.
    let mut target: GlesRenderbuffer = match renderer.create_buffer(Fourcc::Argb8888, buffer_size) {
        Ok(target) => target,
        Err(e) => {
            tracing::warn!("[screenshot] create_buffer failed: {}", e);
            return None;
        }
    };
    tracing::debug!("[screenshot] offscreen buffer created");
    let mut framebuffer = match renderer.bind(&mut target) {
        Ok(fb) => fb,
        Err(e) => {
            tracing::warn!("[screenshot] bind offscreen target failed: {}", e);
            return None;
        }
    };
    tracing::debug!("[screenshot] offscreen target bound");
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
    tracing::debug!("[screenshot] offscreen render done");

    // The renderer's EGL context is already current here (render_output and
    // renderer.bind() make it current), so GL functions can be loaded directly.
    let gl = Gles2::load_with(|symbol| unsafe { get_proc_address(symbol) });
    tracing::debug!("[screenshot] GL functions loaded");
    unsafe {
        gl.ReadBuffer(gl_ffi::COLOR_ATTACHMENT0);
    }
    tracing::debug!("[screenshot] ReadBuffer done");
    let (width, height) = (size.w as usize, size.h as usize);
    let mut rgba = vec![0u8; width * height * 4];
    unsafe {
        gl.ReadPixels(
            0,
            0,
            size.w,
            size.h,
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

    tracing::debug!("[screenshot] pixel conversion done");
    let surface = ImageSurface::create_for_data(bgra, Format::ARgb32, size.w, size.h, row as i32)
        .ok()?;
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

/// Dispatches a desktop notification.
pub fn send_notification(title: &str, body: &str) {
    let _ = std::process::Command::new("notify-send")
        .args(["-a", "BabyDra", title, body])
        .spawn();
}
