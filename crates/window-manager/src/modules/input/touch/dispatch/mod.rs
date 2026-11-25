//! Multi-touch input event handlers: touch down, motion, up, cancel, and frame.

pub(crate) mod gesture_state;

pub(crate) mod overlay;

use smithay::{
    backend::input::{
        InputBackend, TouchCancelEvent, TouchDownEvent, TouchFrameEvent, TouchMotionEvent,
        TouchUpEvent,
    },
    desktop::{layer_map_for_output, PopupManager, Window},
    input::touch::{DownEvent, MotionEvent, UpEvent},
    utils::{Serial, SERIAL_COUNTER},
};

use common::Action;

use crate::modules::input::{
    common::transform_absolute_position, touch::drag, InputContext, MoveState, ResizeState,
};

#[allow(unused_imports)]
pub(crate) mod events;

// ----- touch down -----------------------------------------------------

/// Handles a touchscreen contact press (touch down).
///
/// Raises the touched window, dispatches title-bar actions, and begins
/// resize/move interactions where appropriate.
///
/// # Arguments
/// * `state` - Mutable input context driving the compositor.
/// * `event` - Backend touch-down event carrying slot and position.
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
        raise_and_focus(state, &window, serial);
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
        dismiss_popups(state);
        raise_and_focus(state, &window, serial);
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

/// Raises a window, focuses its toplevel, and activates its element.
///
/// # Arguments
/// * `state` - Mutable input context.
/// * `window` - Window to raise and focus.
/// * `serial` - Serial used for the keyboard focus change.
fn raise_and_focus(state: &mut impl InputContext, window: &Window, serial: Serial) {
    state.space_mut().raise_element(window, true);
    state.raise_window(window);
    if let Some(toplevel) = window.toplevel() {
        state.set_keyboard_focus(Some(toplevel.wl_surface().clone()), serial);
        state.space().elements().for_each(|w| {
            let active = w == window;
            w.set_activated(active);
            if let Some(top) = w.toplevel() {
                top.send_pending_configure();
            }
        });
    }
}

/// Dismisses popups belonging to layer surfaces on every output.
///
/// # Arguments
/// * `state` - Input context providing access to the workspace.
fn dismiss_popups(state: &impl InputContext) {
    for output in state.space().outputs() {
        let layer_map = layer_map_for_output(output);
        for layer in layer_map.layers() {
            let surface = layer.wl_surface();
            for (popup, _) in PopupManager::popups_for_surface(surface) {
                let _ = PopupManager::dismiss_popup(surface, &popup);
            }
        }
    }
}

// ----- touch motion ---------------------------------------------------

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

// ----- touch lifecycle ------------------------------------------------

/// Handles lifting of a finger from the touchscreen surface (touch up).
///
/// # Arguments
/// * `state` - Mutable input context driving the compositor.
/// * `event` - Backend touch-up event carrying the released slot.
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
///
/// # Arguments
/// * `state` - Mutable input context driving the compositor.
/// * `event` - Backend touch-cancel event carrying the cancelled slot.
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
///
/// # Arguments
/// * `state` - Mutable input context driving the compositor.
/// * `_event` - Backend touch-frame event (unused).
pub(crate) fn handle_touch_frame<B: InputBackend, T: TouchFrameEvent<B>>(
    state: &mut impl InputContext,
    _event: T,
) {
    state.touch_frame();
}

// ----- tests ----------------------------------------------------------

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
