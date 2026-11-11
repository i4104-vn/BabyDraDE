//! Interactive window dragging logic during pointer movements.

use smithay::{
    input::pointer::MotionEvent,
    utils::{Point, SERIAL_COUNTER},
};

use crate::compositor::state::BabydraState;

/// Processes active interactive window movement while a pointer button is held down.
///
/// Implements a 4-pixel movement threshold before window dragging officially starts
/// to prevent jitter or accidental micro-moves during clicks.
///
/// # Arguments
///
/// * `state` - Mutable reference to [`BabydraState`].
/// * `position` - Current logical pointer coordinate on screen.
/// * `time` - Timestamp in milliseconds of the motion event.
///
/// # Returns
///
/// Returns `true` if a window drag was active and handled, or `false` to continue normal pointer motion.
pub(crate) fn handle_window_drag(
    state: &mut BabydraState,
    position: Point<f64, smithay::utils::Logical>,
    time: u32,
) -> bool {
    let pointer = state.seat.get_pointer().unwrap();
    if let Some(ref mut move_state) = state.move_state {
        if !state.pointer_pressed_buttons.contains(&move_state.button)
            && state.pointer_pressed_buttons.is_empty()
        {
            state.move_state = None;
            let under = state.surface_under(position);
            pointer.motion(
                state,
                under,
                &MotionEvent {
                    location: position,
                    serial: SERIAL_COUNTER.next_serial(),
                    time,
                },
            );
            pointer.frame(state);
            return true;
        }

        let move_delta = position - move_state.initial_pointer_location;
        let dist_sq = move_delta.x * move_delta.x + move_delta.y * move_delta.y;

        // Require at least a 4-pixel movement threshold before window dragging starts
        if !move_state.has_dragged && dist_sq < 16.0 {
            return true;
        }
        move_state.has_dragged = true;

        let new_loc = move_state.initial_window_location
            + Point::from((move_delta.x.round() as i32, move_delta.y.round() as i32));
        state
            .space
            .map_element(move_state.window.clone(), new_loc, false);
        return true;
    }
    false
}
