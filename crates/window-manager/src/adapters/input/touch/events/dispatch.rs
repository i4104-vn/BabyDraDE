//! Touch event routing: down, motion, up, cancel and frame.

use smithay::{
    backend::input::{
        InputBackend, TouchCancelEvent, TouchDownEvent, TouchFrameEvent, TouchMotionEvent,
        TouchUpEvent,
    },
    input::touch::{DownEvent, MotionEvent, UpEvent},
    utils::SERIAL_COUNTER,
};

use super::{gesture_state, overlay};
use crate::adapters::input::{
    common::transform_absolute_position, InputContext, MoveState, ResizeState,
};
use common::Action;

/// Handles a touchscreen contact press (touch down).
pub(crate) fn handle_touch_down<B: InputBackend, T: TouchDownEvent<B>>(
    state: &mut impl InputContext,
    event: T,
) {
    let Some(position) = transform_absolute_position(state, &event, true) else {
        return;
    };

    state.set_pointer_location(position);
    state.set_cursor_visible(false);
    gesture_state::begin_contact(state, event.slot());

    if overlay::handle_overlay_touch_down(state, event.slot(), position) {
        return;
    }

    // Begin tracking this finger for multi-touch gesture detection.
    gesture_state::track_down(state, event.slot(), position, event.time_msec() as u64);

    let serial = SERIAL_COUNTER.next_serial();

    // 1. Check window border for touch resizing
    if let Some((window, loc, edge)) = state.window_border_under(position) {
        state.space_mut().raise_element(&window, true);
        state.raise_window(&window);
        if let Some(toplevel) = window.toplevel() {
            state.set_keyboard_focus(Some(toplevel.wl_surface().clone()), serial);
            state.space().elements().for_each(|w| {
                let active = w == &window;
                w.set_activated(active);
                if let Some(top) = w.toplevel() {
                    top.send_pending_configure();
                }
            });
        }
        state.update_active_window(Some(&window));
        let size = window.geometry().size;
        *state.resize_state_mut() = Some(ResizeState {
            window,
            initial_window_location: loc,
            initial_window_size: size,
            initial_pointer_location: position,
            edges: edge,
            button: 0,
        });
        return;
    }

    // 2. Check window and titlebar
    if let Some((window, location)) = state.window_under(position) {
        state.space_mut().raise_element(&window, true);
        state.raise_window(&window);
        if let Some(toplevel) = window.toplevel() {
            state.set_keyboard_focus(Some(toplevel.wl_surface().clone()), serial);
            state.space().elements().for_each(|w| {
                let active = w == &window;
                w.set_activated(active);
                if let Some(top) = w.toplevel() {
                    top.send_pending_configure();
                }
            });
        }
        state.update_active_window(Some(&window));

        // Check titlebar action on touch
        if let Some(action) = state.titlebar_action(&window, location, position) {
            match action {
                Action::Close => {
                    *state.last_titlebar_click_mut() = None;
                    state.close_window(&window);
                }
                Action::Minimize => {
                    *state.last_titlebar_click_mut() = None;
                    state.minimize_window(&window);
                }
                Action::Maximize => {
                    *state.last_titlebar_click_mut() = None;
                    state.toggle_maximize(&window);
                }
                Action::Move => {
                    let now = std::time::Instant::now();
                    let is_double_tap = if let Some((last_time, last_pos, ref last_win)) =
                        state.last_titlebar_click_mut().take()
                    {
                        let elapsed = now.duration_since(last_time).as_millis();
                        let dx = position.x - last_pos.x;
                        let dy = position.y - last_pos.y;
                        last_win == &window
                            && elapsed <= common::DOUBLE_CLICK_TIMEOUT_MS
                            && (dx * dx + dy * dy) <= common::DOUBLE_CLICK_DISTANCE_SQ
                    } else {
                        false
                    };

                    if is_double_tap {
                        state.toggle_maximize(&window);
                    } else {
                        *state.last_titlebar_click_mut() = Some((now, position, window.clone()));
                        *state.move_state_mut() = Some(MoveState {
                            window: window.clone(),
                            initial_window_location: location,
                            initial_pointer_location: position,
                            has_dragged: false,
                            button: 0,
                        });
                    }
                }
            }
            return;
        }

        *state.last_titlebar_click_mut() = None;
    }

    let under = state.surface_under(position);
    state.touch_down(
        under,
        &DownEvent {
            slot: event.slot(),
            location: position,
            serial,
            time: event.time_msec(),
        },
    );
    state.touch_frame();
}

/// Handles finger movement across the touchscreen surface.
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

    if super::super::drag::handle_touch_resize(state, position) {
        return;
    }

    if super::super::drag::handle_touch_drag(state, position) {
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

/// Handles lifting of a finger from the touchscreen surface (touch up).
pub(crate) fn handle_touch_up<B: InputBackend, T: TouchUpEvent<B>>(
    state: &mut impl InputContext,
    event: T,
) {
    // Evaluate multi-touch gesture before removing the slot.
    gesture_state::track_up(state, event.slot(), event.time_msec() as u64);

    gesture_state::end_contact(state, event.slot());

    if overlay::handle_overlay_up(state, event.slot()) {
        return;
    }

    if gesture_state::no_contacts(state) {
        gesture_state::finish_interactions(state);
    }

    let serial = SERIAL_COUNTER.next_serial();
    state.touch_up(&UpEvent {
        slot: event.slot(),
        serial,
        time: event.time_msec(),
    });
    state.touch_frame();
}

/// Handles touchscreen contact cancellation (e.g. palm detection or gesture takeover).
pub(crate) fn handle_touch_cancel<B: InputBackend, T: TouchCancelEvent<B>>(
    state: &mut impl InputContext,
    event: T,
) {
    gesture_state::end_contact(state, event.slot());

    if overlay::handle_overlay_cancel(state, event.slot()) {
        return;
    }

    if gesture_state::no_contacts(state) {
        gesture_state::finish_interactions(state);
    }

    state.touch_cancel();
    state.touch_frame();
}

/// Flushes and completes a group of simultaneous multi-touch events.
pub(crate) fn handle_touch_frame<B: InputBackend, T: TouchFrameEvent<B>>(
    state: &mut impl InputContext,
    _event: T,
) {
    state.touch_frame();
}
