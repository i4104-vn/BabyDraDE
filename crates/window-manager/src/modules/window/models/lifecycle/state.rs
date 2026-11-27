use smithay::utils::{Logical, Point, Size};

use super::animation::WindowAnimation;

/// Per-window persistent state attached to [`smithay::desktop::Window`]'s user_data.
#[derive(Default, Debug, Clone)]
pub(crate) struct WindowStateData {
    pub(crate) saved_location: Option<Point<i32, Logical>>,
    pub(crate) saved_size: Option<Size<i32, Logical>>,
    pub(crate) is_minimized: bool,
    pub(crate) has_opened: bool,
    pub(crate) animation: Option<WindowAnimation>,
}
