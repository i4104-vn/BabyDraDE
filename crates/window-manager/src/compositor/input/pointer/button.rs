//! Pointer button press/release handling and window focus activation.

use smithay::{
    backend::input::{ButtonState, InputBackend, PointerButtonEvent},
    input::pointer::{ButtonEvent, MotionEvent},
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::SERIAL_COUNTER,
};

use crate::compositor::state::{BabydraState, MoveState};

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

            if !pointer.is_grabbed() {
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
                            has_dragged: false,
                            button: button_code,
                        });
                        return;
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

