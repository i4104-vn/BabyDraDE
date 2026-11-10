use smithay::{
    backend::input::{
        InputBackend, TouchCancelEvent, TouchDownEvent, TouchFrameEvent, TouchMotionEvent,
        TouchUpEvent,
    },
    input::touch::{DownEvent, MotionEvent, UpEvent},
    utils::{Point, SERIAL_COUNTER},
};

use super::super::state::BabydraState;

pub(crate) fn handle_touch_down<B: InputBackend, T: TouchDownEvent<B>>(
    state: &mut BabydraState,
    event: T,
) {
    let output = state.space.outputs().next();
    let Some(output) = output else { return };
    let Some(output_geometry) = state.space.output_geometry(output) else { return };

    let mut position =
        event.position_transformed(output_geometry.size) + output_geometry.loc.to_f64();
    let max_x = (output_geometry.loc.x + output_geometry.size.w) as f64;
    let max_y = (output_geometry.loc.y + output_geometry.size.h) as f64;
    position.x = position.x.clamp(output_geometry.loc.x as f64, max_x);
    position.y = position.y.clamp(output_geometry.loc.y as f64, max_y);

    state.pointer_location = position;
    state.touch_slots.insert(event.slot());

    let serial = SERIAL_COUNTER.next_serial();
    let keyboard = state.seat.get_keyboard().unwrap();

    if let Some((window, _)) = state.window_under(position) {
        state.space.raise_element(&window, true);
        if let Some(toplevel) = window.toplevel() {
            keyboard.set_focus(
                state,
                Some(toplevel.wl_surface().clone()),
                serial,
            );
            state.space.elements().for_each(|w| {
                if let Some(top) = w.toplevel() {
                    top.send_pending_configure();
                }
            });
        }
        state.update_active_window(Some(&window));
    }

    let under = state.surface_under(position);
    if let Some(touch) = state.seat.get_touch() {
        touch.down(
            state,
            under,
            &DownEvent {
                slot: event.slot(),
                location: position,
                serial,
                time: event.time_msec(),
            },
        );
    }
}

pub(crate) fn handle_touch_motion<B: InputBackend, T: TouchMotionEvent<B>>(
    state: &mut BabydraState,
    event: T,
) {
    let output = state.space.outputs().next();
    let Some(output) = output else { return };
    let Some(output_geometry) = state.space.output_geometry(output) else { return };

    let mut position =
        event.position_transformed(output_geometry.size) + output_geometry.loc.to_f64();
    let max_x = (output_geometry.loc.x + output_geometry.size.w) as f64;
    let max_y = (output_geometry.loc.y + output_geometry.size.h) as f64;
    position.x = position.x.clamp(output_geometry.loc.x as f64, max_x);
    position.y = position.y.clamp(output_geometry.loc.y as f64, max_y);

    state.pointer_location = position;

    // Handle touch-drag window movement if an active move request was started
    if let Some(ref mut move_state) = state.move_state {
        let move_delta = position - move_state.initial_pointer_location;
        let dist_sq = move_delta.x * move_delta.x + move_delta.y * move_delta.y;
        if move_state.has_dragged || dist_sq >= 16.0 {
            move_state.has_dragged = true;
            let new_loc = move_state.initial_window_location
                + Point::from((move_delta.x.round() as i32, move_delta.y.round() as i32));
            state.space.map_element(move_state.window.clone(), new_loc, false);
            return;
        }
    }

    let under = state.surface_under(position);
    if let Some(touch) = state.seat.get_touch() {
        touch.motion(
            state,
            under,
            &MotionEvent {
                slot: event.slot(),
                location: position,
                time: event.time_msec(),
            },
        );
    }
}

pub(crate) fn handle_touch_up<B: InputBackend, T: TouchUpEvent<B>>(
    state: &mut BabydraState,
    event: T,
) {
    state.touch_slots.remove(&event.slot());
    if state.touch_slots.is_empty() {
        state.move_state = None;
    }

    let serial = SERIAL_COUNTER.next_serial();
    if let Some(touch) = state.seat.get_touch() {
        touch.up(
            state,
            &UpEvent {
                slot: event.slot(),
                serial,
                time: event.time_msec(),
            },
        );
    }
}

pub(crate) fn handle_touch_cancel<B: InputBackend, T: TouchCancelEvent<B>>(
    state: &mut BabydraState,
    event: T,
) {
    state.touch_slots.remove(&event.slot());
    if state.touch_slots.is_empty() {
        state.move_state = None;
    }

    if let Some(touch) = state.seat.get_touch() {
        touch.cancel(state);
    }
}

pub(crate) fn handle_touch_frame<B: InputBackend, T: TouchFrameEvent<B>>(
    state: &mut BabydraState,
    _event: T,
) {
    if let Some(touch) = state.seat.get_touch() {
        touch.frame(state);
    }
}
