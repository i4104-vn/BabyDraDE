//! Clipboard handling for the screenshot editor.

use gtk4::prelude::*;
use std::io::Write;

use super::drawing::save_cropped_surface;
use crate::adapters::desktop::screenshot::capture::send_notification;
use crate::adapters::desktop::screenshot::editor::model::EditorState;

/// Copies the cropped annotated screenshot to the clipboard using `wl-copy` or
/// a GTK clipboard fallback. Returns `true` on success.
pub(crate) fn copy_to_clipboard(state: &EditorState, window: &gtk4::ApplicationWindow) -> bool {
    if let Some(mut surface) = save_cropped_surface(state) {
        let temp_copy_path = "/tmp/babydra-screenshot-copy.png";

        let mut png_bytes = Vec::new();
        if surface.write_to_png(&mut png_bytes).is_ok() {
            let _ = std::fs::write(temp_copy_path, &png_bytes);

            let child = std::process::Command::new("wl-copy")
                .args(["-t", "image/png"])
                .stdin(std::process::Stdio::piped())
                .spawn();

            match child {
                Ok(mut c) => {
                    if let Some(mut stdin) = c.stdin.take() {
                        if let Err(err) = stdin.write_all(&png_bytes) {
                            tracing::warn!("[screenshot] cannot feed wl-copy: {err}");
                        }
                    }
                    match c.wait() {
                        Ok(status) if status.success() => {
                            notify_copied();
                            return true;
                        }
                        Ok(status) => tracing::warn!("[screenshot] wl-copy exited: {status}"),
                        Err(err) => tracing::warn!("[screenshot] wl-copy wait failed: {err}"),
                    }
                }
                Err(err) => {
                    // wl-clipboard may be missing; fall back to GTK's clipboard.
                    tracing::warn!("[screenshot] cannot start wl-copy: {err}");
                }
            }
        }

        let w = surface.width();
        let h = surface.height();
        let stride = surface.stride();
        if let Ok(data) = surface.data() {
            let pixbuf = gdk_pixbuf::Pixbuf::from_mut_slice(
                data.to_vec(),
                gdk_pixbuf::Colorspace::Rgb,
                true,
                8,
                w,
                h,
                stride,
            );

            let texture = gtk4::gdk::Texture::for_pixbuf(&pixbuf);
            let clipboard = window.upcast_ref::<gtk4::Widget>().display().clipboard();
            clipboard.set_texture(&texture);

            notify_copied();
            return true;
        }
    }
    false
}

fn notify_copied() {
    let notif_title = services::trans("screenshot.copied_title");
    let notif_msg = services::trans("screenshot.copied_msg");
    send_notification(&notif_title, &notif_msg);
}
