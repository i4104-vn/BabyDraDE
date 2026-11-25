//! Cached rendering data for the editor canvas.

use std::cell::RefCell;
use std::rc::Rc;

/// Cached static background. Only the screenshot itself is cached: the crop
/// rectangle, dimming and annotations are drawn live so moving the crop never
/// triggers an expensive full-surface rebuild (which showed up as a ~1s stall
/// before the drag preview appeared).
pub(crate) struct LayerCache {
    pub(crate) width: i32,
    pub(crate) height: i32,
    pub(crate) scale: i32,
    pub(crate) surface: cairo::ImageSurface,
}

/// Handle shared between the draw function and gesture controllers.
pub(crate) type SharedCache = Rc<RefCell<Option<LayerCache>>>;
