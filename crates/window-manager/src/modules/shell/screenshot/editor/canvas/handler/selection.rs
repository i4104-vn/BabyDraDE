//! Drawing hit-testing and selection helpers.

use crate::modules::shell::screenshot::editor::models::EditorState;

/// Selects the topmost drawing under (`px`, `py`), returning `true` when one was hit.
pub(super) fn pick_drawing(s: &mut EditorState, px: f64, py: f64) -> bool {
    let found = s.drawings.iter().rposition(|d| d.hit_test(px, py));
    let changed = s.selected_drawing != found;
    s.selected_drawing = found;
    if changed {
        s.invalidate();
    }
    found.is_some()
}

/// Removes the currently selected drawing, keeping indices consistent.
pub(crate) fn delete_selected(s: &mut EditorState) {
    if let Some(idx) = s.selected_drawing.take() {
        if idx < s.drawings.len() {
            s.drawings.remove(idx);
            s.invalidate();
        }
    }
}

/// Erases the topmost drawing under (`x`, `y`).
pub(super) fn erase_at(s: &mut EditorState, x: f64, y: f64) {
    if let Some(idx) = s.drawings.iter().rposition(|d| d.hit_test(x, y)) {
        s.drawings.remove(idx);
        s.selected_drawing = match s.selected_drawing {
            Some(sel) if sel > idx => Some(sel - 1),
            Some(sel) if sel == idx => None,
            other => other,
        };
        s.invalidate();
    }
}
