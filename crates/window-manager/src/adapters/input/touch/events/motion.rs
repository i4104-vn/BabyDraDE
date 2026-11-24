//! Touch motion handling.

use smithay::{
    backend::input::{InputBackend, TouchMotionEvent},
    input::touch::MotionEvent,
};

use crate::adapters::input::{
    common::transform_absolute_position,
    touch::{
        drag,
        events::{gesture_state, overlay},
    },
    InputContext,
};

/// Handles finger movement across the touchscreen surface.
///
/// Updates gesture tracking, routes overlay/drag interactions, and forwards
/// the motion to the surface beneath the finger.
///
/// # Arguments
/// * `state` - Mutable input context driving the compositor.
/// * `event` - Backend touch-motion event carrying slot and position.
pub(crate) fn handle_touch_motion<B: InputBackend, T: TouchMotionEvent<B>>(
    state: &mut impl InputContext,
    event: T,
) {
    let Some(position) = transform_absolute_position(state, &event, true) else {
        return;
    };

    state.set_pointer_location(position);
    state.set_cursor_visible(false);

    // Update gesture tracking position for this slot.
    gesture_state::track_motion(state, event.slot(), position);

    if overlay::handle_overlay_motion(state, event.slot(), position) {
        return;
    }

    if drag::handle_touch_resize(state, position) {
        return;
    }

    if drag::handle_touch_drag(state, position) {
        return;
    }

    let under = state.surface_under(position);
    state.touch_motion(
        under,
        &MotionEvent {
            slot: event.slot(),
            location: position,
            time: event.time_msec(),
        },
    );
    state.touch_frame();
}
