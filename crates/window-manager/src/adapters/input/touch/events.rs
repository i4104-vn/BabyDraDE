//! Multi-touch input event handlers: touch down, motion, up, cancel, and frame.

use smithay::{
    backend::input::{
        InputBackend, TouchCancelEvent, TouchDownEvent, TouchFrameEvent, TouchMotionEvent,
        TouchUpEvent,
    },
    input::touch::{DownEvent, MotionEvent, UpEvent},
    utils::SERIAL_COUNTER,
};

use super::super::InputContext;
use super::drag::{handle_touch_drag, handle_touch_resize};
use crate::adapters::input::touch::gestures::{on_touch_down, on_touch_motion, on_touch_up};
use crate::adapters::input::{
    common::transform_absolute_position, MoveState, OverlayTouchState, ResizeState,
};
use common::Action;

/// Checks whether a touch-down occurs on an active overlay (launcher or switcher)
/// and registers tracking state instead of immediately launching/activating.
fn handle_overlay_touch_down<C: InputContext>(
    state: &mut C,
    slot: smithay::backend::input::TouchSlot,
    position: smithay::utils::Point<f64, smithay::utils::Logical>,
) -> bool {
    if !state.switcher_is_active() && state.launcher_is_visible() {
        *state.overlay_touch_mut() = Some(OverlayTouchState {
            slot,
            start_pos: position,
            last_pos: position,
            has_dragged: false,
            is_switcher: false,
        });
        return true;
    }

    if state.switcher_is_active() {
        *state.overlay_touch_mut() = Some(OverlayTouchState {
            slot,
            start_pos: position,
            last_pos: position,
            has_dragged: false,
            is_switcher: true,
        });
        return true;
    }

    false
}

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
    state.touch_slots_mut().insert(event.slot());

    if handle_overlay_touch_down(state, event.slot(), position) {
        return;
    }

    // Begin tracking this finger for multi-touch gesture detection.
    on_touch_down(state, event.slot(), position, event.time_msec() as u64);

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
    on_touch_motion(state, event.slot(), position);

    if let Some(mut touch) = state.overlay_touch_mut().take() {
        if touch.slot == event.slot() {
            let dy = position.y - touch.last_pos.y;
            let total_dy = (position.y - touch.start_pos.y).abs();
            touch.last_pos = position;

            if total_dy > 8.0 || touch.has_dragged {
                touch.has_dragged = true;
                let slots = -dy / common::constants::launcher::SLOT;
                if touch.is_switcher {
                    state.switcher_drag_by(slots);
                } else if state.launcher_is_visible() {
                    state.launcher_drag_by(slots);
                }
            } else if !touch.is_switcher && state.launcher_is_visible() {
                state.launcher_update_hover(position);
            }

            *state.overlay_touch_mut() = Some(touch);
            return;
        } else {
            *state.overlay_touch_mut() = Some(touch);
        }
    }

    if handle_touch_resize(state, position) {
        return;
    }

    if handle_touch_drag(state, position) {
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
    on_touch_up(state, event.slot(), event.time_msec() as u64);

    state.touch_slots_mut().remove(&event.slot());

    if let Some(touch) = state.overlay_touch_mut().take() {
        if touch.slot == event.slot() {
            if touch.has_dragged {
                // Drag completed: snap to nearest slot
                if touch.is_switcher {
                    state.switcher_finish_drag();
                } else if state.launcher_is_visible() {
                    state.launcher_finish_drag();
                }
            } else {
                // Tap gesture: launch app or select window
                if touch.is_switcher {
                    let serial = SERIAL_COUNTER.next_serial();
                    let slide = state.switcher_slide(common::constants::launcher::PANEL_WIDTH);
                    let tapped = state
                        .switcher_index_at((touch.start_pos.x, touch.start_pos.y), slide)
                        .and_then(|index| state.switcher_window_at(index));

                    if let Some(window) = tapped {
                        state.unminimize_window(&window);
                        crate::adapters::input::keyboard::focus::focus_window(
                            state, window, serial,
                        );
                    }
                    state.switcher_begin_close();
                } else if state.launcher_is_visible() {
                    if let Some(index) = state.launcher_index_at(touch.start_pos) {
                        state.launcher_launch(index);
                    } else {
                        state.launcher_close();
                    }
                }
            }
            return;
        } else {
            *state.overlay_touch_mut() = Some(touch);
        }
    }

    if state.touch_slots().is_empty() {
        if let Some(move_state) = state.move_state_mut().take() {
            if move_state.has_dragged {
                if let Some(toplevel) = move_state.window.toplevel() {
                    toplevel.send_configure();
                }
            }
        }
        if let Some(resize_state) = state.resize_state_mut().take() {
            if let Some(toplevel) = resize_state.window.toplevel() {
                toplevel.with_pending_state(|state| {
                    state.states.unset(
                        smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::State::Resizing,
                    );
                });
                toplevel.send_configure();
            }
        }
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
    state.touch_slots_mut().remove(&event.slot());

    if let Some(touch) = state.overlay_touch_mut().take() {
        if touch.slot == event.slot() {
            if touch.has_dragged {
                if touch.is_switcher {
                    state.switcher_finish_drag();
                } else if state.launcher_is_visible() {
                    state.launcher_finish_drag();
                }
            }
            return;
        } else {
            *state.overlay_touch_mut() = Some(touch);
        }
    }

    if state.touch_slots().is_empty() {
        if let Some(move_state) = state.move_state_mut().take() {
            if move_state.has_dragged {
                if let Some(toplevel) = move_state.window.toplevel() {
                    toplevel.send_configure();
                }
            }
        }
        if let Some(resize_state) = state.resize_state_mut().take() {
            if let Some(toplevel) = resize_state.window.toplevel() {
                toplevel.with_pending_state(|state| {
                    state.states.unset(
                        smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::State::Resizing,
                    );
                });
                toplevel.send_configure();
            }
        }
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
