//! Multi-touch input event handlers: touch down, motion, up, cancel, and frame.

use smithay::{
    backend::input::{
        InputBackend, TouchCancelEvent, TouchDownEvent, TouchFrameEvent, TouchMotionEvent,
        TouchUpEvent,
    },
    input::touch::{DownEvent, MotionEvent, UpEvent},
    utils::SERIAL_COUNTER,
};

use crate::compositor::input::common::transform_absolute_position;
use crate::compositor::input::touch::gestures::{on_touch_down, on_touch_motion, on_touch_up};
use crate::compositor::state::BabydraState;
use super::drag::handle_touch_drag;

/// Handles a touchscreen contact press (touch down).
///
/// Hides the software cursor during direct finger manipulation, raises and focuses
/// the touched window, tracks the touch slot, and sends `wl_touch.down` to clients.
///
/// # Arguments
///
/// * `state` - Mutable reference to [`BabydraState`].
/// * `event` - The touch down event providing coordinates, slot index, and timestamp.
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

    // Begin tracking this finger for multi-touch gesture detection.
    on_touch_down(state, event.slot(), position, event.time_msec() as u64);

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

/// Handles finger movement across the touchscreen surface.
///
/// Drives touch-based window dragging if active, or delivers motion frames to the client.
///
/// # Arguments
///
/// * `state` - Mutable reference to [`BabydraState`].
/// * `event` - The touch motion event providing updated coordinates and slot index.
pub(crate) fn handle_touch_motion<B: InputBackend, T: TouchMotionEvent<B>>(
    state: &mut BabydraState,
    event: T,
) {
    let Some(position) = transform_absolute_position(state, &event, true) else {
        return;
    };

    state.pointer_location = position;
    state.cursor_visible = false;

    // Update gesture tracking position for this slot.
    on_touch_motion(state, event.slot(), position);

    if handle_touch_drag(state, position) {
        return;
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

/// Handles lifting of a finger from the touchscreen surface (touch up).
///
/// Cleans up tracked touch slots and move state, and sends `wl_touch.up` to the client.
///
/// # Arguments
///
/// * `state` - Mutable reference to [`BabydraState`].
/// * `event` - The touch up event providing slot index and timestamp.
pub(crate) fn handle_touch_up<B: InputBackend, T: TouchUpEvent<B>>(
    state: &mut BabydraState,
    event: T,
) {
    // Evaluate multi-touch gesture before removing the slot.
    on_touch_up(state, event.slot(), event.time_msec() as u64);

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

/// Handles touchscreen contact cancellation (e.g. palm detection or gesture takeover).
///
/// Cancels active touch points and resets tracking state.
///
/// # Arguments
///
/// * `state` - Mutable reference to [`BabydraState`].
/// * `event` - The touch cancel event providing slot index.
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

/// Flushes and completes a group of simultaneous multi-touch events.
///
/// # Arguments
///
/// * `state` - Mutable reference to [`BabydraState`].
/// * `_event` - The touch frame event marking end of current contact batch.
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

