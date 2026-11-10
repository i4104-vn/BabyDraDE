use smithay::{
    backend::input::{
        InputBackend, TouchCancelEvent, TouchDownEvent, TouchFrameEvent, TouchMotionEvent,
        TouchUpEvent,
    },
    input::touch::{DownEvent, MotionEvent, UpEvent},
    utils::{Point, SERIAL_COUNTER},
};

use super::super::state::BabydraState;
use super::transform_absolute_position;

pub(crate) fn handle_touch_down<B: InputBackend, T: TouchDownEvent<B>>(
    state: &mut BabydraState,
    event: T,
) {
    let Some(position) = transform_absolute_position(state, &event, true) else {
        return;
    };

    state.pointer_location = position;
    state.cursor_visible = false;
    state.touch_slots.insert(event.slot());

    let serial = SERIAL_COUNTER.next_serial();
    let keyboard = state.seat.get_keyboard().unwrap();

    if let Some((window, _)) = state.window_under(position) {
        state.space.raise_element(&window, true);
        state.raise_window(&window);
        if let Some(toplevel) = window.toplevel() {
            keyboard.set_focus(
                state,
                Some(toplevel.wl_surface().clone()),
                serial,
            );
            state.space.elements().for_each(|w| {
                let active = w == &window;
                w.set_activated(active);
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
        touch.frame(state);
    }
}

pub(crate) fn handle_touch_motion<B: InputBackend, T: TouchMotionEvent<B>>(
    state: &mut BabydraState,
    event: T,
) {
    let Some(position) = transform_absolute_position(state, &event, true) else {
        return;
    };

    state.pointer_location = position;
    state.cursor_visible = false;

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
        touch.frame(state);
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
        touch.frame(state);
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
        touch.frame(state);
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

#[cfg(test)]
mod tests {
    use smithay::utils::{Logical, Point, Size, Transform};

    #[test]
    fn test_touch_coordinate_rotation_90() {
        let raw_size = Size::<f64, Logical>::from((1600.0, 2560.0));
        let transform = Transform::_90;

        // Physical center (800, 1280) -> Landscape center (1280, 800)
        let pt_center = Point::<f64, Logical>::from((800.0, 1280.0));
        let oriented_center = transform.transform_point_in(pt_center, &raw_size);
        assert_eq!(oriented_center.x, 1280.0);
        assert_eq!(oriented_center.y, 800.0);

        // Physical bottom-left (0, 2560) -> Landscape top-left (0, 0)
        let pt_bl = Point::<f64, Logical>::from((0.0, 2560.0));
        let oriented_bl = transform.transform_point_in(pt_bl, &raw_size);
        assert_eq!(oriented_bl.x, 0.0);
        assert_eq!(oriented_bl.y, 0.0);

        // Physical top-left (0, 0) -> Landscape top-right (2560, 0)
        let pt_tl = Point::<f64, Logical>::from((0.0, 0.0));
        let oriented_tl = transform.transform_point_in(pt_tl, &raw_size);
        assert_eq!(oriented_tl.x, 2560.0);
        assert_eq!(oriented_tl.y, 0.0);
    }
}
