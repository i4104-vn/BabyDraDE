//! Pointer motion handling for relative and absolute pointing devices.

use smithay::{
    backend::input::{AbsolutePositionEvent, InputBackend, PointerMotionEvent},
    input::pointer::MotionEvent,
    utils::SERIAL_COUNTER,
};

use super::drag::handle_window_drag;
use crate::compositor::input::common::transform_absolute_position;
use crate::compositor::state::BabydraState;

/// Handles relative pointer motion events (such as mice and touchpad cursor movements).
///
/// Updates the compositor pointer location clamped to screen boundaries, drives active
/// window drags, updates pointer focus, and delivers motion frames to client surfaces.
///
/// # Arguments
///
/// * `state` - Mutable reference to [`BabydraState`].
/// * `event` - The relative pointer motion event containing delta x and y.
pub(crate) fn handle_pointer_motion<B: InputBackend, P: PointerMotionEvent<B>>(
    state: &mut BabydraState,
    event: P,
) {
    state.cursor_visible = true;
    let pointer = state.seat.get_pointer().unwrap();
    let delta = event.delta();
    let mut position = state.pointer_location + delta;

    if let Some(output) = state.space.outputs().next() {
        if let Some(geo) = state.space.output_geometry(output) {
            let max_x = (geo.loc.x + geo.size.w) as f64;
            let max_y = (geo.loc.y + geo.size.h) as f64;
            position.x = position.x.clamp(geo.loc.x as f64, max_x);
            position.y = position.y.clamp(geo.loc.y as f64, max_y);
        }
    }

    state.pointer_location = position;

    if handle_window_drag(state, position, event.time_msec()) {
        return;
    }

    let under = state.surface_under(position);
    pointer.motion(
        state,
        under,
        &MotionEvent {
            location: position,
            serial: SERIAL_COUNTER.next_serial(),
            time: event.time_msec(),
        },
    );
    pointer.frame(state);
}

/// Handles absolute pointer motion events (such as drawing tablets or virtual VM pointers).
///
/// Converts hardware absolute coordinates to logical coordinates using output geometry,
/// clamps to screen boundaries, updates window drags, and forwards motion to Wayland clients.
///
/// # Arguments
///
/// * `state` - Mutable reference to [`BabydraState`].
/// * `event` - The absolute position event providing coordinates.
pub(crate) fn handle_pointer_motion_absolute<B: InputBackend, P: AbsolutePositionEvent<B>>(
    state: &mut BabydraState,
    event: P,
) {
    state.cursor_visible = true;
    let Some(position) = transform_absolute_position(state, &event, false) else {
        return;
    };
    let pointer = state.seat.get_pointer().unwrap();

    state.pointer_location = position;

    if handle_window_drag(state, position, event.time_msec()) {
        return;
    }

    let under = state.surface_under(position);
    pointer.motion(
        state,
        under,
        &MotionEvent {
            location: position,
            serial: SERIAL_COUNTER.next_serial(),
            time: event.time_msec(),
        },
    );
    pointer.frame(state);
}
