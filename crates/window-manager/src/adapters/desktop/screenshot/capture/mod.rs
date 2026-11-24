//! Direct compositor screenshot capture — no external capture utilities.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

/// Shared flag set by the keymap thread and consumed by the render loops.
static CAPTURE_REQUESTED: AtomicBool = AtomicBool::new(false);

/// Resolves the default file path to save screenshots in `~/Pictures/Screenshots/`.
pub(crate) fn get_screenshot_path() -> PathBuf {
    let pictures_dir = dirs::picture_dir().unwrap_or_else(|| {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        home.join("Pictures")
    });
    let screenshots_dir = pictures_dir.join("Screenshots");
    let _ = std::fs::create_dir_all(&screenshots_dir);

    let datetime = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    screenshots_dir.join(format!("Screenshot_{}.png", datetime))
}

/// Converts smithay's `ReadPixels` output into cairo's `ARgb32` buffer.
///
/// Smithay's GLES projection includes a `flip180` factor, so rows arrive
/// top-down already; do **not** reverse them or the saved PNG ends up mirrored
/// (and rotated incorrectly on transformed outputs). Only RGBA -> BGRA needs
/// swapping on little-endian hosts.
fn rgba_to_argb32(rgba: &[u8], width: usize, height: usize) -> Vec<u8> {
    let mut bgra = vec![0u8; rgba.len()];
    let row = width * 4;
    for y in 0..height {
        let src = &rgba[y * row..(y + 1) * row];
        let dst = &mut bgra[y * row..(y + 1) * row];
        for x in 0..width {
            let (s, d) = (&src[x * 4..x * 4 + 4], &mut dst[x * 4..x * 4 + 4]);
            d[0] = s[2];
            d[1] = s[1];
            d[2] = s[0];
            d[3] = s[3];
        }
    }
    bgra
}

/// Requests a screenshot. Safe to call from any thread.
pub(crate) fn trigger() {
    CAPTURE_REQUESTED.store(true, Ordering::SeqCst);
}

/// Consumes a pending screenshot request. Called once per rendered frame.
pub(crate) fn take_requested() -> bool {
    CAPTURE_REQUESTED.swap(false, Ordering::SeqCst)
}

/// Dispatches a desktop notification.
pub(crate) fn send_notification(title: &str, body: &str) {
    let _ = std::process::Command::new("notify-send")
        .args(["-a", "BabyDra", title, body])
        .spawn();
}


pub(crate) use render::capture_screen;

mod render;
mod transform;

#[cfg(test)]
mod tests;
