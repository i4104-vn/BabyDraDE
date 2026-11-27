//! Dimming and crop outline overlays drawn every frame.

use crate::modules::shell::screenshot::editor::models::EditorState;

/// Dimming layer. With a crop selection the hole is cut out with the EvenOdd
/// fill rule, otherwise the whole screenshot is dimmed. Drawn every frame so
/// it tracks the pointer without touching the cache.
pub(super) fn draw_dimming(
    cr: &cairo::Context,
    s: &EditorState,
    width: f64,
    height: f64,
    has_clip: bool,
) {
    if let Err(err) = cr.save() {
        tracing::warn!("cannot save context for dimming: {err}");
        return;
    }
    cr.set_source_rgba(0.0, 0.0, 0.0, 0.45);
    cr.rectangle(0.0, 0.0, width, height);
    if has_clip {
        cr.rectangle(s.crop_x, s.crop_y + s.crop_h, s.crop_w, -s.crop_h);
        cr.set_fill_rule(cairo::FillRule::EvenOdd);
    }
    if let Err(err) = cr.fill() {
        tracing::warn!("cannot draw dimming layer: {err}");
    }
    if let Err(err) = cr.restore() {
        tracing::warn!("cannot restore context after dimming: {err}");
    }
}

/// Blue outline of the crop rectangle.
pub(super) fn draw_crop_border(cr: &cairo::Context, s: &EditorState) {
    if !s.has_selection {
        return;
    }
    if let Err(err) = cr.save() {
        tracing::warn!("cannot save context for crop border: {err}");
        return;
    }
    cr.set_source_rgba(0.23, 0.51, 0.96, 0.85);
    cr.set_line_width(2.0);
    cr.rectangle(s.crop_x, s.crop_y, s.crop_w, s.crop_h);
    if let Err(err) = cr.stroke() {
        tracing::warn!("cannot draw crop border: {err}");
    }
    if let Err(err) = cr.restore() {
        tracing::warn!("cannot restore context after crop border: {err}");
    }
}
