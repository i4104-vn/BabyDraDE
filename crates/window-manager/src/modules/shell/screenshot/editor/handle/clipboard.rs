//! Clipboard handling for the screenshot editor.

use gtk4::prelude::*;
use services::screenshot::{copy_to_clipboard as copy_png, notify};

use super::drawing::save_cropped_surface;
use crate::modules::shell::screenshot::editor::models::EditorState;

/// Copies the cropped annotated screenshot to the clipboard using `wl-copy` or
/// a GTK clipboard fallback. Returns `true` on success.
pub(crate) fn copy_to_clipboard(state: &EditorState, window: &gtk4::ApplicationWindow) -> bool {
    if let Some(mut surface) = save_cropped_surface(state) {
        let mut png_bytes = Vec::new();
        if surface.write_to_png(&mut png_bytes).is_ok() && copy_png(&png_bytes) {
            notify_copied();
            return true;
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
    notify(&notif_title, &notif_msg);
}
