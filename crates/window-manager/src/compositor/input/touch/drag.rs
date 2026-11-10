//! Touchscreen interactive window movement handling.

use smithay::utils::Point;

use crate::compositor::state::BabydraState;

/// Processes active interactive window movement driven by touchscreen dragging.
///
/// Ensures a minimum distance threshold (16 square pixels) before beginning window translation.
///
/// # Arguments
///
/// * `state` - Mutable reference to [`BabydraState`].
/// * `position` - Current logical touch coordinate on screen.
///
/// # Returns
///
/// Returns `true` if a window drag was active and handled, or `false` to continue normal touch motion.
pub(crate) fn handle_touch_drag(
    state: &mut BabydraState,
    position: Point<f64, smithay::utils::Logical>,
) -> bool {
    if let Some(ref mut move_state) = state.move_state {
        let move_delta = position - move_state.initial_pointer_location;
        let dist_sq = move_delta.x * move_delta.x + move_delta.y * move_delta.y;
        if move_state.has_dragged || dist_sq >= 16.0 {
            move_state.has_dragged = true;
            let new_loc = move_state.initial_window_location
                + Point::from((move_delta.x.round() as i32, move_delta.y.round() as i32));
            state.space.map_element(move_state.window.clone(), new_loc, false);
            return true;
        }
    }
    false
}

