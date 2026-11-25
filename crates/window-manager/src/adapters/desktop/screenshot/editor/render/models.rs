//! Data contracts shared by the editor window and its toolbar builders.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::adapters::desktop::screenshot::editor::models::EditorState;

/// Widgets and state shared by toolbar button handlers.
pub(super) struct ToolbarContext {
    /// Editor state mutated by the toolbar actions.
    pub(super) state: Rc<RefCell<EditorState>>,
    /// Editor window closed by the action buttons.
    pub(super) window: gtk4::ApplicationWindow,
    /// Canvas queued for redraw after a reset.
    pub(super) canvas: gtk4::DrawingArea,
    /// Wrapper whose visibility is toggled by the reset action.
    pub(super) wrapper: gtk4::Box,
    /// Animation generation counter to support clean cancellation of slide-in/out.
    pub(super) anim_generation: Rc<Cell<u64>>,
}

/// Widgets produced by the toolbar builder that the window assembly needs.
pub(super) struct ToolbarParts {
    /// The assembled action bar container holding the tool capsule and right action buttons.
    pub(super) action_bar: gtk4::Box,
    /// Pen button used by canvas gestures to restore the active tool.
    pub(super) pen: gtk4::Button,
    /// Color popover opened by right-clicking the canvas.
    pub(super) color_popover: gtk4::Popover,
}
