use std::collections::HashMap;

use smithay::{
    desktop::Window,
    reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::ResizeEdge,
    utils::{Logical, Point, Size},
};

#[derive(Debug, Clone)]
pub(crate) struct MoveState {
    pub(crate) window: Window,
    pub(crate) initial_window_location: Point<i32, Logical>,
    pub(crate) initial_pointer_location: Point<f64, Logical>,
    pub(crate) has_dragged: bool,
    pub(crate) button: u32,
}

#[derive(Debug, Clone)]
pub(crate) struct ResizeState {
    pub(crate) window: Window,
    pub(crate) initial_window_location: Point<i32, Logical>,
    pub(crate) initial_window_size: Size<i32, Logical>,
    pub(crate) initial_pointer_location: Point<f64, Logical>,
    pub(crate) edges: ResizeEdge,
    pub(crate) button: u32,
}

impl ResizeState {
    pub(crate) fn resizes_left(&self) -> bool {
        matches!(
            self.edges,
            ResizeEdge::Left | ResizeEdge::TopLeft | ResizeEdge::BottomLeft
        )
    }
    pub(crate) fn resizes_right(&self) -> bool {
        matches!(
            self.edges,
            ResizeEdge::Right | ResizeEdge::TopRight | ResizeEdge::BottomRight
        )
    }
    pub(crate) fn resizes_top(&self) -> bool {
        matches!(
            self.edges,
            ResizeEdge::Top | ResizeEdge::TopLeft | ResizeEdge::TopRight
        )
    }
    pub(crate) fn resizes_bottom(&self) -> bool {
        matches!(
            self.edges,
            ResizeEdge::Bottom | ResizeEdge::BottomLeft | ResizeEdge::BottomRight
        )
    }
}

#[derive(Debug, Clone)]
pub(crate) struct TrackpadSwipeBuffer {
    pub(crate) fingers: u32,
    pub(crate) time: u32,
    pub(crate) total_delta: Point<f64, Logical>,
    pub(crate) updates: Vec<(Point<f64, Logical>, u32)>,
    /// 3-finger horizontal drag is scrubbing the switcher selection.
    pub(crate) scrubbing: bool,
    pub(crate) scrub_start_x: f64,
    pub(crate) scrub_base_index: usize,
}

/// Per-slot data accumulated while tracking a multi-touch gesture on a touchscreen.
#[derive(Debug, Clone)]
pub(crate) struct TouchSlotData {
    /// Starting position when the finger first touched down.
    pub(crate) start: Point<f64, Logical>,
    /// Most recent position of this finger.
    pub(crate) current: Point<f64, Logical>,
    /// Wall-clock millisecond timestamp at first contact.
    pub(crate) start_time_ms: u64,
}

/// In-progress multi-touch gesture being accumulated for touchscreen swipe/tap detection.
#[derive(Debug, Clone, Default)]
pub(crate) struct TouchGestureBuffer {
    /// Live per-slot tracking data keyed by slot index.
    pub(crate) slots: HashMap<smithay::backend::input::TouchSlot, TouchSlotData>,
    /// Whether the gesture has already been consumed (action was fired).
    pub(crate) consumed: bool,
    /// 3-finger horizontal drag is scrubbing the switcher selection.
    pub(crate) scrubbing: bool,
    pub(crate) scrub_start_x: f64,
    pub(crate) scrub_base_index: usize,
}

/// Active single-finger touch interaction with compositor overlays (launcher or switcher).
#[derive(Debug, Clone)]
pub(crate) struct OverlayTouchState {
    pub(crate) slot: smithay::backend::input::TouchSlot,
    pub(crate) start_pos: Point<f64, Logical>,
    pub(crate) last_pos: Point<f64, Logical>,
    pub(crate) has_dragged: bool,
    pub(crate) is_switcher: bool,
}
