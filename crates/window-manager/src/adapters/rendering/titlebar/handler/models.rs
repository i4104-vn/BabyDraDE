//! Geometry and appearance data for one titlebar frame.

use common::constants::{TITLEBAR_BUTTON_RADIUS, TITLEBAR_HEIGHT};
use common::Action;
use smithay::utils::{Logical, Point};

use crate::domain::geometry::Point as PurePoint;
use crate::domain::TitlebarLayout;

/// Converts a Smithay location to the pure point expected by the layout rule.
///
/// # Arguments
///
/// * `location` - Window location in logical coordinates.
///
/// # Returns
///
/// The equivalent framework-free point.
pub(super) fn to_pure_location(location: Point<i32, Logical>) -> PurePoint<i32> {
    PurePoint::from((location.x, location.y))
}

/// Sampled appearance used to paint the titlebar background.
pub(super) struct PaintStyle {
    /// Background color sampled from the client.
    pub(super) color: [f64; 3],
    /// Whether translucency should be used.
    pub(super) should_blur: bool,
    /// Translucency level when blur is enabled.
    pub(super) opacity: f32,
    /// Whether the top corners should stay square.
    pub(super) maximized: bool,
}

/// Button centers and pointer hover state for one frame.
pub(super) struct Controls {
    /// Close button center.
    pub(super) close_x: f64,
    /// Maximize button center.
    pub(super) max_x: f64,
    /// Minimize button center.
    pub(super) min_x: f64,
    /// Shared vertical center of all buttons.
    pub(super) cy: f64,
    /// Whether the pointer is over the close button.
    pub(super) hover_close: bool,
    /// Whether the pointer is over the maximize button.
    pub(super) hover_max: bool,
    /// Whether the pointer is over the minimize button.
    pub(super) hover_min: bool,
    /// Whether the pointer is inside the button cluster.
    pub(super) in_cluster: bool,
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
    pub(super) fn new(
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
