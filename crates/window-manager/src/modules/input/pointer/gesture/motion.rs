//! Pointer motion handling for relative and absolute pointing devices.

use smithay::{
    backend::input::{AbsolutePositionEvent, InputBackend, PointerMotionEvent},
    input::pointer::MotionEvent,
    utils::{Logical, Point, SERIAL_COUNTER},
};

use super::drag::{handle_window_drag, handle_window_resize};
use crate::modules::input::transform::transform_absolute_position;
use crate::modules::input::InputContext;

fn update_overlay_hover<C: InputContext>(state: &mut C, position: Point<f64, Logical>) {
    if !state.switcher_is_active() {
        state.launcher_update_hover(position);
        return;
    }

    let slide = state.switcher_slide(common::constants::launcher::PANEL_WIDTH);
    let index = state.switcher_index_at((position.x, position.y), slide);
    state.switcher_set_hover(index);
}

/// Handles relative pointer motion events (such as mice and touchpad cursor movements).
pub(crate) fn handle_pointer_motion<B: InputBackend, P: PointerMotionEvent<B>>(
    state: &mut impl InputContext,
    event: P,
) {
    state.set_cursor_visible(true);
    let delta = event.delta();
    let mut position = state.pointer_location() + delta;

    let output = state.space().outputs().next().cloned();
    if let Some(output) = output.as_ref() {
        if let Some(geo) = state.space().output_geometry(output) {
            let max_x = (geo.loc.x + geo.size.w) as f64;
            let max_y = (geo.loc.y + geo.size.h) as f64;
            position.x = position.x.clamp(geo.loc.x as f64, max_x);
            position.y = position.y.clamp(geo.loc.y as f64, max_y);
        }
    }

    state.set_pointer_location(position);

    update_overlay_hover(state, position);

    if handle_window_resize(state, position) {
        return;
    }

    if handle_window_drag(state, position, event.time_msec()) {
        return;
    }

    let under = state.surface_under(position);
    state.pointer_motion(
        under,
        &MotionEvent {
            location: position,
            serial: SERIAL_COUNTER.next_serial(),
            time: event.time_msec(),
        },
    );
    state.pointer_frame();
}

/// Handles absolute pointer motion events (such as drawing tablets or virtual VM pointers).
pub(crate) fn handle_absolute_motion<B: InputBackend, P: AbsolutePositionEvent<B>>(
    state: &mut impl InputContext,
    event: P,
) {
    state.set_cursor_visible(true);
    let Some(position) = transform_absolute_position(state, &event, false) else {
        return;
    };

    state.set_pointer_location(position);

    update_overlay_hover(state, position);

    if handle_window_resize(state, position) {
        return;
    }

    if handle_window_drag(state, position, event.time_msec()) {
        return;
    }

    let under = state.surface_under(position);
    state.pointer_motion(
        under,
        &MotionEvent {
            location: position,
            serial: SERIAL_COUNTER.next_serial(),
            time: event.time_msec(),
        },
    );
    state.pointer_frame();
}
