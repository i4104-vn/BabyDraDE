//! Switcher layout geometry models.

use ui_kit::components::dock::DockLayout;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Rect {
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) width: f64,
    pub(crate) height: f64,
}

impl Rect {
    pub(crate) fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width: width.max(1.0),
            height: height.max(1.0),
        }
    }
}

pub(crate) struct SwitcherLayout {
    /// Shared right-edge capsule.
    pub(crate) panel: DockLayout,
    /// App currently highlighted (pointer or keyboard).
    pub(crate) selected: usize,
    /// Small live preview beside the active icon.
    pub(crate) preview: Rect,
    /// Fade of the preview while switching.
    pub(crate) preview_alpha: f64,
}
