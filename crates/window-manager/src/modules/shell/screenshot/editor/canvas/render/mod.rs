//! Canvas rendering assembly: cached background plus live editing overlays.

mod annotations;
mod cache;
mod overlays;
mod preview;

use crate::modules::shell::screenshot::editor::models::EditorState;

use super::SharedCache;
use annotations::render_annotations;
use cache::{blit_cache, refresh_cache};
use overlays::{draw_crop_border, draw_dimming};
use preview::render_preview;

/// Draws the cached background plus everything that changes while editing.
pub(crate) fn draw_editor_canvas(
    cr: &cairo::Context,
    s: &EditorState,
    width: f64,
    height: f64,
    cache: &SharedCache,
    scale_factor: i32,
) {
    refresh_cache(
        s,
        width.max(1.0) as i32,
        height.max(1.0) as i32,
        scale_factor,
        cache,
    );
    blit_cache(cr, cache);

    let has_clip = s.has_selection && s.crop_w > 5.0 && s.crop_h > 5.0;

    // Live dimming (with the crop hole) and outline — no cache rebuild.
    draw_dimming(cr, s, width, height, has_clip);
    draw_crop_border(cr, s);

    // Finished annotations, clipped to the crop selection when present.
    cr.set_antialias(cairo::Antialias::Best);
    if has_clip {
        cr.save().unwrap();
        cr.rectangle(s.crop_x, s.crop_y, s.crop_w, s.crop_h);
        cr.clip();
    }
    render_annotations(cr, s);
    if has_clip {
        cr.restore().unwrap();
    }

    // Live preview of the shape currently being dragged.
    cr.set_antialias(cairo::Antialias::None);
    if has_clip {
        cr.save().unwrap();
        cr.rectangle(s.crop_x, s.crop_y, s.crop_w, s.crop_h);
        cr.clip();
    }
    render_preview(cr, s);
    if has_clip {
        cr.restore().unwrap();
    }
}
