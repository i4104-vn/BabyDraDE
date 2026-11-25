//! Screenshot editor runtime state.

use super::drawing::{Drawing, Tool, STROKE_WIDTHS};

/// Current active state of the editor.
pub(crate) struct EditorState {
    pub(crate) bg_pixbuf: gdk_pixbuf::Pixbuf,
    pub(crate) crop_x: f64,
    pub(crate) crop_y: f64,
    pub(crate) crop_w: f64,
    pub(crate) crop_h: f64,
    pub(crate) canvas_w: f64,
    pub(crate) canvas_h: f64,
    pub(crate) has_selection: bool,
    pub(crate) drag_start_x: f64,
    pub(crate) drag_start_y: f64,
    pub(crate) is_selecting: bool,
    pub(crate) current_tool: Tool,
    /// Last shape picked in the shapes popover; applied on first click of the
    /// shared shapes button so drawing starts immediately.
    pub(crate) current_shape: Tool,
    pub(crate) current_color: (f64, f64, f64),
    pub(crate) current_width: f64,
    pub(crate) drawings: Vec<Drawing>,
    pub(crate) selected_drawing: Option<usize>,
    pub(crate) active_stroke: Option<Vec<(f64, f64)>>,
    pub(crate) active_rect: Option<(f64, f64, f64, f64)>,
    pub(crate) active_line: Option<(f64, f64, f64, f64)>,
    /// Incremented whenever static content (background/crop/drawings) changes so the
    /// canvas render cache can detect invalidation cheaply.
    pub(crate) render_version: u64,
}

impl EditorState {
    /// Creates a new editor state with the provided raw background pixbuf.
    pub(crate) fn new(pixbuf: gdk_pixbuf::Pixbuf) -> Self {
        Self {
            bg_pixbuf: pixbuf,
            crop_x: 0.0,
            crop_y: 0.0,
            crop_w: 0.0,
            crop_h: 0.0,
            canvas_w: 0.0,
            canvas_h: 0.0,
            has_selection: false,
            drag_start_x: 0.0,
            drag_start_y: 0.0,
            is_selecting: false,
            current_tool: Tool::Select,
            current_shape: Tool::Rect,
            current_color: (0.93, 0.15, 0.15),
            current_width: STROKE_WIDTHS[1],
            drawings: Vec::new(),
            selected_drawing: None,
            active_stroke: None,
            active_rect: None,
            active_line: None,
            render_version: 0,
        }
    }

    /// Marks static content as changed so cached renders are invalidated.
    pub(crate) fn invalidate(&mut self) {
        self.render_version += 1;
    }
}
