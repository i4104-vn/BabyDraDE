use common::constants::{TITLEBAR_BUTTON_RADIUS, TITLEBAR_HEIGHT};
use common::Action;
use smithay::utils::{Logical, Point};

use crate::shared::types::TitlebarLayout;

use crate::modules::window::titlebar::paint::to_pure_location;

/// Sampled appearance used to paint the titlebar background.
pub(crate) struct PaintStyle {
    /// Background color sampled from the client.
    pub(crate) color: [f64; 3],
    /// Whether translucency should be used.
    pub(crate) should_blur: bool,
    /// Translucency level when blur is enabled.
    pub(crate) opacity: f32,
    /// Whether the top corners should stay square.
    pub(crate) maximized: bool,
}

/// Button centers and pointer hover state for one frame.
pub(crate) struct Controls {
    /// Close button center.
    pub(crate) close_x: f64,
    /// Maximize button center.
    pub(crate) max_x: f64,
    /// Minimize button center.
    pub(crate) min_x: f64,
    /// Shared vertical center of all buttons.
    pub(crate) cy: f64,
    /// Whether the pointer is over the close button.
    pub(crate) hover_close: bool,
    /// Whether the pointer is over the maximize button.
    pub(crate) hover_max: bool,
    /// Whether the pointer is over the minimize button.
    pub(crate) hover_min: bool,
    /// Whether the pointer is inside the button cluster.
    pub(crate) in_cluster: bool,
}

impl Controls {
    /// Computes button centers and hover flags from the current pointer.
    ///
    /// # Arguments
    ///
    /// * `layout` - Titlebar geometry for this window width.
    /// * `location` - Window location in logical coordinates.
    /// * `pointer` - Pointer position in global logical coordinates.
    /// * `maximized` - Whether the window is maximized.
    ///
    /// # Returns
    ///
    /// The button centers and their hover state.
    pub(crate) fn new(
        layout: &TitlebarLayout,
        location: Point<i32, Logical>,
        pointer: Point<f64, Logical>,
        maximized: bool,
    ) -> Self {
        let cy = layout.button(Action::Close).y;
        let close_x = layout.button(Action::Close).x;
        let max_x = layout.button(Action::Maximize).x;
        let min_x = layout.button(Action::Minimize).x;

        let frame_y = layout.frame_y(to_pure_location(location), maximized);
        let local_pointer =
            Point::<f64, Logical>::from((pointer.x - location.x as f64, pointer.y - frame_y));

        let in_cluster = local_pointer.x >= (min_x - TITLEBAR_BUTTON_RADIUS)
            && local_pointer.x <= (close_x + TITLEBAR_BUTTON_RADIUS)
            && local_pointer.y >= 0.0
            && local_pointer.y <= TITLEBAR_HEIGHT as f64;

        let hover_close =
            (local_pointer.x - close_x).hypot(local_pointer.y - cy) <= TITLEBAR_BUTTON_RADIUS;
        let hover_max =
            (local_pointer.x - max_x).hypot(local_pointer.y - cy) <= TITLEBAR_BUTTON_RADIUS;
        let hover_min =
            (local_pointer.x - min_x).hypot(local_pointer.y - cy) <= TITLEBAR_BUTTON_RADIUS;

        Self {
            close_x,
            max_x,
            min_x,
            cy,
            hover_close,
            hover_max,
            hover_min,
            in_cluster,
        }
    }
}
