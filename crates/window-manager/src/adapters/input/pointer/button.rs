//! Pointer button press/release handling and window focus activation.

use smithay::{
    backend::input::{ButtonState, InputBackend, PointerButtonEvent},
    desktop::{layer_map_for_output, PopupManager},
    input::pointer::{ButtonEvent, MotionEvent},
    utils::SERIAL_COUNTER,
};

use common::Action;

use super::super::InputContext;
use crate::adapters::input::{MoveState, ResizeState};

/// Handles pointer button clicks, focus switching, window raising, and Super+Drag initiation.
pub(crate) fn handle_pointer_button<B: InputBackend, P: PointerButtonEvent<B>>(
    state: &mut impl InputContext,
    event: P,
) {
    state.set_cursor_visible(true);
    let serial = SERIAL_COUNTER.next_serial();
    let button_code = event.button_code();

    match event.state() {
        ButtonState::Pressed => {
            state.pointer_pressed_buttons_mut().insert(button_code);

            if state.switcher_is_active() {
                let px = state.pointer_location().x;
                let py = state.pointer_location().y;

                if let Some(clicked_idx) = state.switcher_index_at((px, py), 0.0) {
                    if let Some(window) = state.switcher_window_at(clicked_idx) {
                        state.unminimize_window(&window);
                        crate::adapters::input::keyboard::focus::focus_window(
                            state, window, serial,
                        );
                    }
                    state.switcher_begin_close();
                    return;
                }
            }

            // While the search overlay is expanded it owns the whole screen:
            // clicking a result activates it, clicking outside returns to the
            // docked launcher.
            if button_code == common::BTN_LEFT
                && !state.pointer_is_grabbed()
                && state.launcher_search_click(state.pointer_location())
            {
                return;
            }

            // The launcher is compositor-owned UI: a click on one of its icons
            // launches that app and must not reach (or raise) any window.
            if !state.switcher_is_active()
                && button_code == common::BTN_LEFT
                && !state.pointer_is_grabbed()
            {
                if let Some(index) = state.launcher_index_at(state.pointer_location()) {
                    state.launcher_launch(index);
                    return;
                }
                // Clicking elsewhere dismisses the launcher.
                if state.launcher_is_visible() {
                    state.launcher_toggle();
                }
            }

            if !state.pointer_is_grabbed() {
                // 1. Check window border for interactive resize
                if button_code == common::BTN_LEFT {
                    if let Some((window, loc, edge)) =
                        state.window_border_under(state.pointer_location())
                    {
                        state.space_mut().raise_element(&window, true);
                        state.raise_window(&window);
                        state.set_keyboard_focus(
                            Some(window.toplevel().unwrap().wl_surface().clone()),
                            serial,
                        );
                        state.space().elements().for_each(|w| {
                            let active = w == &window;
                            w.set_activated(active);
                            if let Some(top) = w.toplevel() {
                                top.send_pending_configure();
                            }
                        });
                        state.update_active_window(Some(&window));
                        let size = window.geometry().size;
                        *state.resize_state_mut() = Some(ResizeState {
                            window,
                            initial_window_location: loc,
                            initial_window_size: size,
                            initial_pointer_location: state.pointer_location(),
                            edges: edge,
                            button: button_code,
                        });
                        return;
                    }
                }

                if let Some((window, location)) = state.window_under(state.pointer_location()) {
                    // Clicking an application window dismisses any open layer popups (e.g. context menus)
                    for output in state.space().outputs() {
                        let layer_map = layer_map_for_output(output);
                        for layer in layer_map.layers() {
                            let surface = layer.wl_surface();
                            for (popup, _) in PopupManager::popups_for_surface(surface) {
                                let _ = PopupManager::dismiss_popup(surface, &popup);
                            }
                        }
                    }

                    state.space_mut().raise_element(&window, true);
                    state.raise_window(&window);
                    state.set_keyboard_focus(
                        Some(window.toplevel().unwrap().wl_surface().clone()),
                        serial,
                    );
                    state.space().elements().for_each(|w| {
                        let active = w == &window;
                        w.set_activated(active);
                        if let Some(top) = w.toplevel() {
                            top.send_pending_configure();
                        }
                    });
                    state.update_active_window(Some(&window));

                    if let Some(action) =
                        state.titlebar_action(&window, location, state.pointer_location())
                    {
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
                                let is_double_click =
                                    if let Some((last_time, last_pos, ref last_win)) =
                                        state.last_titlebar_click_mut().take()
                                    {
                                        let elapsed = now.duration_since(last_time).as_millis();
                                        let dx = state.pointer_location().x - last_pos.x;
                                        let dy = state.pointer_location().y - last_pos.y;
                                        last_win == &window
                                            && elapsed <= common::DOUBLE_CLICK_TIMEOUT_MS
                                            && (dx * dx + dy * dy)
                                                <= common::DOUBLE_CLICK_DISTANCE_SQ
                                    } else {
                                        false
                                    };

                                if is_double_click {
                                    state.toggle_maximize(&window);
                                } else {
                                    *state.last_titlebar_click_mut() =
                                        Some((now, state.pointer_location(), window.clone()));
                                    *state.move_state_mut() = Some(MoveState {
                                        window: window.clone(),
                                        initial_window_location: location,
                                        initial_pointer_location: state.pointer_location(),
                                        has_dragged: false,
                                        button: button_code,
                                    });
                                }
                            }
                        }
                        return;
                    }

                    *state.last_titlebar_click_mut() = None;

                    // Ensure pointer focus is updated to the raised window surface
                    let under = state.surface_under(state.pointer_location());
                    state.pointer_motion(
                        under,
                        &MotionEvent {
                            location: state.pointer_location(),
                            serial,
                            time: event.time_msec(),
                        },
                    );
                    state.pointer_frame();

                    // Drag or resize window from anywhere if holding Super (Win) key
                    let mods = state.keyboard_modifiers();
                    if mods.logo {
                        if button_code == common::BTN_RIGHT {
                            let size = window.geometry().size;
                            *state.resize_state_mut() = Some(ResizeState {
                                window: window.clone(),
                                initial_window_location: location,
                                initial_window_size: size,
                                initial_pointer_location: state.pointer_location(),
                                edges: smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::ResizeEdge::BottomRight,
                                button: button_code,
                            });
                            return;
                        } else {
                            *state.move_state_mut() = Some(MoveState {
                                window: window.clone(),
                                initial_window_location: location,
                                initial_pointer_location: state.pointer_location(),
                                has_dragged: false,
                                button: button_code,
                            });
                            return;
                        }
                    }
                } else {
                    let clicked_layer = state.layer_under(state.pointer_location());
                    let is_top_or_overlay = matches!(
                        clicked_layer,
                        Some(smithay::wayland::shell::wlr_layer::Layer::Top)
                            | Some(smithay::wayland::shell::wlr_layer::Layer::Overlay)
                    );

                    // Ensure pointer focus is updated to the clicked surface
                    let under = state.surface_under(state.pointer_location());
                    state.pointer_motion(
                        under,
                        &MotionEvent {
                            location: state.pointer_location(),
                            serial,
                            time: event.time_msec(),
                        },
                    );
                    state.pointer_frame();

                    // When interacting with panels, topbar, or overlays, do NOT lose active app focus
                    if !is_top_or_overlay {
                        for output in state.space().outputs() {
                            let layer_map = layer_map_for_output(output);
                            for layer in layer_map.layers() {
                                let surface = layer.wl_surface();
                                for (popup, _) in PopupManager::popups_for_surface(surface) {
                                    let _ = PopupManager::dismiss_popup(surface, &popup);
                                }
                            }
                        }

                        state.space().elements().for_each(|window| {
                            window.set_activated(false);
                            window.toplevel().unwrap().send_pending_configure();
                        });
                        state.set_keyboard_focus(None, serial);
                        state.update_active_window(None);
                    }
                }
            }
        }
        ButtonState::Released => {
            state.pointer_pressed_buttons_mut().remove(&button_code);

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

            if let Some(move_state) = state.move_state_mut().take() {
                if move_state.has_dragged {
                    if let Some(toplevel) = move_state.window.toplevel() {
                        toplevel.send_configure();
                    }
                }
                let under = state.surface_under(state.pointer_location());
                state.pointer_motion(
                    under,
                    &MotionEvent {
                        location: state.pointer_location(),
                        serial,
                        time: event.time_msec(),
                    },
                );
                state.pointer_frame();
            }
        }
    }

    state.pointer_button(&ButtonEvent {
        button: button_code,
        state: event.state(),
        serial,
        time: event.time_msec(),
    });
    state.pointer_frame();
}
