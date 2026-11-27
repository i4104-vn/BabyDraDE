//! Editor window UI construction: overlay layout, glassmorphic toolbar, and canvas.

mod buttons;
mod color;
mod toolbar;
mod tools;
mod window;

pub(crate) use crate::modules::shell::screenshot::models::{ToolbarContext, ToolbarParts};
pub use window::build_editor_ui;
