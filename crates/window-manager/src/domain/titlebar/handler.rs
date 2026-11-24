//! Titlebar hit-test entry point.
//!
//! Converts a frame snapshot into a pure layout decision.

use super::models::{FrameInput, TitlebarLayout};
use crate::domain::geometry::Point;
use common::Action;

/// Hit-tests a pointer position against a compositor-owned frame.
///
/// # Arguments
///
/// * `frame` - Frame geometry and decoration state.
/// * `point` - Pointer position in global logical coordinates.
///
/// # Returns
///
/// The action under the pointer, or `None` when the point is outside a frame
/// that can be hit tested.
pub(crate) fn hit_test(frame: FrameInput, point: Point<f64>) -> Option<Action> {
    if !frame.server_side_decoration || frame.fullscreen {
        return None;
    }
    TitlebarLayout::new(frame.width, frame.maximized).hit_test(
        frame.location,
        frame.width,
        frame.maximized,
        point,
    )
}
