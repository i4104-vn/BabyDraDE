//! Filesystem, notification, and clipboard operations for screenshots.

use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

/// Scratch file mirroring PNG data before it is handed to `wl-copy`.
const CLIPBOARD_TEMP_PATH: &str = "/tmp/babydra-screenshot-copy.png";

/// Resolves the default screenshot path under `~/Pictures/Screenshots/`.
///
/// The directory is created if it does not exist. When neither the pictures
/// nor the home directory is available, `/tmp/Pictures` is used as a fallback.
///
/// # Returns
/// A path of the form `Screenshot_<YYYY-MM-DD_HH-MM-SS>.png`.
pub fn get_screenshot_path() -> PathBuf {
    let pictures_dir = dirs::picture_dir().unwrap_or_else(|| {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        home.join("Pictures")
    });
    screenshot_path_in(&pictures_dir, &timestamp())
}

/// Builds a timestamped path inside `pictures_root/Screenshots`, creating the
/// directory when needed. Kept separate so path handling can be tested without
/// touching the user's home directory.
fn screenshot_path_in(pictures_root: &Path, stamp: &str) -> PathBuf {
    let dir = pictures_root.join("Screenshots");
    let _ = fs::create_dir_all(&dir);
    dir.join(format!("Screenshot_{stamp}.png"))
}

/// Local timestamp using the same layout as `%Y-%m-%d_%H-%M-%S`.
fn timestamp() -> String {
    glib::DateTime::now_local()
        .and_then(|date| date.format("%Y-%m-%d_%H-%M-%S"))
        .map(|text| text.to_string())
        .unwrap_or_default()
}

/// Encodes a composed image as a PNG file.
///
/// # Arguments
/// * `surface` - fully composited image to write.
/// * `path` - destination file; an existing file is truncated.
///
/// # Returns
/// `true` when the PNG was written, `false` on creation or encoding errors.
pub fn save_png(surface: &cairo::ImageSurface, path: &Path) -> bool {
    let mut file = match File::create(path) {
        Ok(file) => file,
        Err(err) => {
            tracing::warn!("[screenshot] cannot create {}: {}", path.display(), err);
            return false;
        }
    };
    if let Err(err) = surface.write_to_png(&mut file) {
        tracing::warn!("[screenshot] cannot write PNG: {}", err);
        return false;
    }
    true
}

/// Sends a desktop notification through `notify-send` without waiting.
///
/// # Arguments
/// * `title` - notification title.
/// * `body` - notification text.
pub fn notify(title: &str, body: &str) {
    let _ = Command::new("notify-send")
        .args(["-a", "BabyDra", title, body])
        .spawn();
}

/// Copies PNG data to the Wayland clipboard through `wl-copy`.
///
/// The bytes are also mirrored to `/tmp/babydra-screenshot-copy.png` as a
/// debugging aid, matching the legacy editor behavior. Callers remain
/// responsible for any GTK clipboard fallback when this returns `false`.
///
/// # Arguments
/// * `png_bytes` - encoded PNG image to copy.
///
/// # Returns
/// `true` only when `wl-copy` accepted the data and exited successfully.
pub fn copy_to_clipboard(png_bytes: &[u8]) -> bool {
    let _ = fs::write(CLIPBOARD_TEMP_PATH, png_bytes);

    let mut child = match Command::new("wl-copy")
        .args(["-t", "image/png"])
        .stdin(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(err) => {
            tracing::warn!("[screenshot] cannot start wl-copy: {err}");
            return false;
        }
    };

    if let Some(mut stdin) = child.stdin.take() {
        if let Err(err) = stdin.write_all(png_bytes) {
            tracing::warn!("[screenshot] cannot feed wl-copy: {err}");
        }
    }
    match child.wait() {
        Ok(status) if status.success() => true,
        Ok(status) => {
            tracing::warn!("[screenshot] wl-copy exited: {status}");
            false
        }
        Err(err) => {
            tracing::warn!("[screenshot] wl-copy wait failed: {err}");
            false
        }
    }
}

#[cfg(test)]
mod tests;
