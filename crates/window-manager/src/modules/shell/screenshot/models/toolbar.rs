//! Data contracts shared by the editor window and its toolbar builders.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::modules::shell::screenshot::editor::models::EditorState;

/// Widgets and state shared by toolbar button handlers.
pub(crate) struct ToolbarContext {
    /// Editor state mutated by the toolbar actions.
    pub(crate) state: Rc<RefCell<EditorState>>,
    /// Editor window closed by the action buttons.
    pub(crate) window: gtk4::ApplicationWindow,
    /// Canvas queued for redraw after a reset.
    pub(crate) canvas: gtk4::DrawingArea,
    /// Wrapper whose visibility is toggled by the reset action.
    pub(crate) wrapper: gtk4::Box,
    /// Animation generation counter to support clean cancellation of slide-in/out.
    pub(crate) anim_generation: Rc<Cell<u64>>,
}

/// Widgets produced by the toolbar builder that the window assembly needs.
pub(crate) struct ToolbarParts {
    /// The assembled action bar container holding the tool capsule and right action buttons.
    pub(crate) action_bar: gtk4::Box,
    /// Pen button used by canvas gestures to restore the active tool.
    pub(crate) pen: gtk4::Button,
    /// Color popover opened by right-clicking the canvas.
    pub(crate) color_popover: gtk4::Popover,
}
