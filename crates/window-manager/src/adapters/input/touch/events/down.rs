//! Touch contact press handling (touch down).

use smithay::{
    backend::input::{InputBackend, TouchDownEvent},
    desktop::{layer_map_for_output, PopupManager, Window},
    input::touch::DownEvent,
    utils::{SERIAL_COUNTER, Serial},
};

use common::Action;

use crate::adapters::input::{
    common::transform_absolute_position,
    touch::events::{gesture_state, overlay},
    InputContext, MoveState, ResizeState,
};

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
