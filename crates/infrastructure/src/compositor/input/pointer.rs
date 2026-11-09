use smithay::{
    backend::input::{
        AbsolutePositionEvent, ButtonState, InputBackend, PointerButtonEvent, PointerMotionEvent,
    },
    input::pointer::{ButtonEvent, MotionEvent},
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Point, SERIAL_COUNTER},
};

use super::super::state::{BabydraState, MoveState};

pub(crate) fn handle_pointer_motion<B: InputBackend, P: PointerMotionEvent<B>>(
    state: &mut BabydraState,
    event: P,
) {
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

    // Handle window dragging/moving
    if let Some(ref move_state) = state.move_state {
        let move_delta = position - move_state.initial_pointer_location;
        let new_loc = move_state.initial_window_location
            + Point::from((move_delta.x.round() as i32, move_delta.y.round() as i32));
        state.space.map_element(move_state.window.clone(), new_loc, false);
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

pub(crate) fn handle_pointer_motion_absolute<B: InputBackend, P: AbsolutePositionEvent<B>>(
    state: &mut BabydraState,
    event: P,
) {
    let output = state.space.outputs().next().unwrap();
    let output_geometry = state.space.output_geometry(output).unwrap();
    let position =
        event.position_transformed(output_geometry.size) + output_geometry.loc.to_f64();
    let pointer = state.seat.get_pointer().unwrap();

    state.pointer_location = position;

    // Handle window dragging/moving
    if let Some(ref move_state) = state.move_state {
        let move_delta = position - move_state.initial_pointer_location;
        let new_loc = move_state.initial_window_location
            + Point::from((move_delta.x.round() as i32, move_delta.y.round() as i32));
        state.space.map_element(move_state.window.clone(), new_loc, false);
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

pub(crate) fn handle_pointer_button<B: InputBackend, P: PointerButtonEvent<B>>(
    state: &mut BabydraState,
    event: P,
) {
    let pointer = state.seat.get_pointer().unwrap();
    let keyboard = state.seat.get_keyboard().unwrap();
    let serial = SERIAL_COUNTER.next_serial();

    match event.state() {
        ButtonState::Pressed if !pointer.is_grabbed() => {
            if let Some((window, location)) = state.window_under(state.pointer_location) {
                state.space.raise_element(&window, true);
                keyboard.set_focus(
                    state,
                    Some(window.toplevel().unwrap().wl_surface().clone()),
                    serial,
                );
                state.space.elements().for_each(|window| {
                    window.toplevel().unwrap().send_pending_configure();
                });

                // Ensure pointer focus is updated to the raised window surface
                let under = state.surface_under(state.pointer_location);
                pointer.motion(
                    state,
                    under,
                    &MotionEvent {
                        location: state.pointer_location,
                        serial,
                        time: event.time_msec(),
                    },
                );
                pointer.frame(state);

                // Drag window from anywhere if holding Super (Win) key
                let mods = keyboard.modifier_state();
                if mods.logo {
                    state.move_state = Some(MoveState {
                        window: window.clone(),
                        initial_window_location: location,
                        initial_pointer_location: state.pointer_location,
                    });
                    return;
                }
            } else {
                state.space.elements().for_each(|window| {
                    window.set_activated(false);
                    window.toplevel().unwrap().send_pending_configure();
                });
                keyboard.set_focus(state, Option::<WlSurface>::None, serial);
            }
        }
        ButtonState::Released => {
            if state.move_state.take().is_some() {
                let under = state.surface_under(state.pointer_location);
                pointer.motion(
                    state,
                    under,
                    &MotionEvent {
                        location: state.pointer_location,
                        serial,
                        time: event.time_msec(),
                    },
                );
                pointer.frame(state);
            }
        }
        _ => {}
    }

    pointer.button(
        state,
        &ButtonEvent {
            button: event.button_code(),
            state: event.state(),
            serial,
            time: event.time_msec(),
        },
    );
    pointer.frame(state);
}
