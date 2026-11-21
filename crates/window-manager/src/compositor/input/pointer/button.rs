//! Pointer button press/release handling and window focus activation.

use smithay::{
    backend::input::{ButtonState, InputBackend, PointerButtonEvent},
    input::pointer::{ButtonEvent, MotionEvent},
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::SERIAL_COUNTER,
};

use crate::compositor::state::{BabydraState, MoveState};
use crate::rendering::titlebar::Action;

/// Handles pointer button clicks, focus switching, window raising, and Super+Drag initiation.
///
/// When a mouse button is pressed:
/// - Raises the clicked window and assigns keyboard focus.
/// - If the Super (Logo/Win) key is held down, begins interactive window dragging.
/// - Preserves application focus when clicking desktop panels or overlays.
/// - Dispatches the button event and frame to Wayland client surfaces.
///
/// # Arguments
///
/// * `state` - Mutable reference to [`BabydraState`].
/// * `event` - The pointer button event containing button code, press/release state, and timestamp.
pub(crate) fn handle_pointer_button<B: InputBackend, P: PointerButtonEvent<B>>(
    state: &mut BabydraState,
    event: P,
) {
    state.cursor_visible = true;
    let pointer = state.seat.get_pointer().unwrap();
    let keyboard = state.seat.get_keyboard().unwrap();
    let serial = SERIAL_COUNTER.next_serial();
    let button_code = event.button_code();

    match event.state() {
        ButtonState::Pressed => {
            state.pointer_pressed_buttons.insert(button_code);

            if let Some(switcher) = state.alt_tab_switcher.as_ref() {
                let output = state.space.outputs().next().cloned();
                let (size, scale) = output
                    .as_ref()
                    .and_then(|o| {
                        let mode = o.current_mode()?;
                        let transform = o.current_transform();
                        let size = transform.transform_size(mode.size);
                        let scale = o.current_scale().fractional_scale();
                        Some((size, smithay::utils::Scale::from(scale)))
                    })
                    .unwrap_or(((1920, 1080).into(), smithay::utils::Scale::from(1.0)));

                let logical_w = (size.w as f64 / scale.x).max(1.0);
                let logical_h = (size.h as f64 / scale.y).max(1.0);

                if let Some(layout) = crate::extensions::switcher::view::compute_layout(
                    switcher, logical_w, logical_h,
                ) {
                    let px = state.pointer_location.x;
                    let py = state.pointer_location.y;
                    let in_panel = px >= layout.panel.x
                        && px <= layout.panel.x + layout.panel.width
                        && py >= layout.panel.y
                        && py <= layout.panel.y + layout.panel.height;
                    let clicked_idx = if in_panel {
                        layout
                            .dock_items
                            .iter()
                            .find(|(_, item)| {
                                py >= item.card.y && py <= item.card.y + item.card.height
                            })
                            .map(|(idx, _)| *idx)
                    } else {
                        None
                    };

                    let clicked_window = clicked_idx.and_then(|idx| {
                        state.alt_tab_switcher.as_ref().and_then(|switcher| {
                            switcher.entries.get(idx).map(|entry| entry.window.clone())
                        })
                    });
                    if let Some(window) = clicked_window {
                        state.unminimize_window(&window);
                        crate::compositor::input::keyboard::focus::focus_window(
                            state, window, serial,
                        );
                    }
                    if let Some(switcher) = state.alt_tab_switcher.as_mut() {
                        switcher.begin_close();
                    }
                    return;
                }
            }

            // The launcher is compositor-owned UI: a click on one of its icons
            // launches that app and must not reach (or raise) any window.
            if state.alt_tab_switcher.is_none()
                && button_code == common::BTN_LEFT
                && !pointer.is_grabbed()
            {
                if let Some(index) = state.launcher.index_at(state.pointer_location) {
                    state.launcher.launch(index);
                    return;
                }
                // Clicking elsewhere dismisses the launcher.
                if state.launcher.is_visible() {
                    state.launcher.toggle();
                }
            }

            if !pointer.is_grabbed() {
                // 1. Check window border for interactive resize
                if button_code == common::BTN_LEFT {
                    if let Some((window, loc, edge)) =
                        state.window_border_under(state.pointer_location)
                    {
                        state.space.raise_element(&window, true);
                        state.raise_window(&window);
                        keyboard.set_focus(
                            state,
                            Some(window.toplevel().unwrap().wl_surface().clone()),
                            serial,
                        );
                        state.space.elements().for_each(|w| {
                            let active = w == &window;
                            w.set_activated(active);
                            if let Some(top) = w.toplevel() {
                                top.send_pending_configure();
                            }
                        });
                        state.update_active_window(Some(&window));
                        let size = window.geometry().size;
                        state.resize_state = Some(crate::compositor::state::ResizeState {
                            window,
                            initial_window_location: loc,
                            initial_window_size: size,
                            initial_pointer_location: state.pointer_location,
                            edges: edge,
                            button: button_code,
                        });
                        return;
                    }
                }

                if let Some((window, location)) = state.window_under(state.pointer_location) {
                    state.space.raise_element(&window, true);
                    state.raise_window(&window);
                    keyboard.set_focus(
                        state,
                        Some(window.toplevel().unwrap().wl_surface().clone()),
                        serial,
                    );
                    state.space.elements().for_each(|w| {
                        let active = w == &window;
                        w.set_activated(active);
                        if let Some(top) = w.toplevel() {
                            top.send_pending_configure();
                        }
                    });
                    state.update_active_window(Some(&window));

                    if let Some(action) = crate::rendering::titlebar::hit_test(
                        &window,
                        location,
                        state.pointer_location,
                    ) {
                        match action {
                            Action::Close => {
                                state.last_titlebar_click = None;
                                state.close_window(&window);
                            }
                            Action::Minimize => {
                                state.last_titlebar_click = None;
                                state.minimize_window(&window);
                            }
                            Action::Maximize => {
                                state.last_titlebar_click = None;
                                state.toggle_maximize(&window);
                            }
                            Action::Move => {
                                let now = std::time::Instant::now();
                                let is_double_click =
                                    if let Some((last_time, last_pos, ref last_win)) =
                                        state.last_titlebar_click.take()
                                    {
                                        let elapsed = now.duration_since(last_time).as_millis();
                                        let dx = state.pointer_location.x - last_pos.x;
                                        let dy = state.pointer_location.y - last_pos.y;
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
                                    state.last_titlebar_click =
                                        Some((now, state.pointer_location, window.clone()));
                                    state.move_state = Some(MoveState {
                                        window: window.clone(),
                                        initial_window_location: location,
                                        initial_pointer_location: state.pointer_location,
                                        has_dragged: false,
                                        button: button_code,
                                    });
                                }
                            }
                        }
                        return;
                    }

                    state.last_titlebar_click = None;

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

                    // Drag or resize window from anywhere if holding Super (Win) key
                    let mods = keyboard.modifier_state();
                    if mods.logo {
                        if button_code == common::BTN_RIGHT {
                            let size = window.geometry().size;
                            state.resize_state = Some(crate::compositor::state::ResizeState {
                                window: window.clone(),
                                initial_window_location: location,
                                initial_window_size: size,
                                initial_pointer_location: state.pointer_location,
                                edges: smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::ResizeEdge::BottomRight,
                                button: button_code,
                            });
                            return;
                        } else {
                            state.move_state = Some(MoveState {
                                window: window.clone(),
                                initial_window_location: location,
                                initial_pointer_location: state.pointer_location,
                                has_dragged: false,
                                button: button_code,
                            });
                            return;
                        }
                    }
                } else {
                    let clicked_layer = state.layer_under(state.pointer_location);
                    let is_top_or_overlay = matches!(
                        clicked_layer,
                        Some(smithay::wayland::shell::wlr_layer::Layer::Top)
                            | Some(smithay::wayland::shell::wlr_layer::Layer::Overlay)
                    );

                    // When interacting with panels, topbar, or overlays, do NOT lose active app focus
                    if !is_top_or_overlay {
                        state.space.elements().for_each(|window| {
                            window.set_activated(false);
                            window.toplevel().unwrap().send_pending_configure();
                        });
                        keyboard.set_focus(state, Option::<WlSurface>::None, serial);
                        state.update_active_window(None);
                    }
                }
            }
        }
        ButtonState::Released => {
            state.pointer_pressed_buttons.remove(&button_code);

            if let Some(resize_state) = state.resize_state.take() {
                if let Some(toplevel) = resize_state.window.toplevel() {
                    toplevel.with_pending_state(|state| {
                        state.states.unset(
                            smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::State::Resizing,
                        );
                    });
                    toplevel.send_configure();
                }
            }

            if let Some(move_state) = state.move_state.take() {
                if move_state.has_dragged {
                    if let Some(toplevel) = move_state.window.toplevel() {
                        toplevel.send_configure();
                    }
                }
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
    }

    pointer.button(
        state,
        &ButtonEvent {
            button: button_code,
            state: event.state(),
            serial,
            time: event.time_msec(),
        },
    );
    pointer.frame(state);
}
