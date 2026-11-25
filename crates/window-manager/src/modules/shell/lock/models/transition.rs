//! Progressive blur lock transition state.

use std::cell::Cell;
use std::rc::Rc;

#[derive(Clone)]
pub(crate) struct LockTransition {
    pub(crate) stack: gtk4::Stack,
    pub(crate) window: gtk4::ApplicationWindow,
    pub(crate) cancelled: Rc<Cell<bool>>,
    pub(crate) last_blur_frame: Rc<Cell<usize>>,
}
