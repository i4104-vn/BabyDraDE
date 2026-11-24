use smithay::{
    delegate_xdg_shell,
    desktop::{PopupKind, PopupManager, Space, Window},
    reexports::wayland_server::protocol::{wl_seat, wl_surface::WlSurface},
    utils::{Serial, SERIAL_COUNTER},
    wayland::{
        compositor::with_states,
        shell::xdg::{
            PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState,
            XdgToplevelSurfaceData,
        },
    },
};

use super::super::state::BabydraState;

impl XdgShellHandler for BabydraState {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.wayland.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        let window = Window::new_wayland_window(surface.clone());
        crate::adapters::window::metadata::refresh(&window);
        let count = self.windows.space.elements().count() as i32;
        let offset = (count * 36) % 288;
        let loc = (48 + offset, 48 + offset);
        self.windows.space.map_element(window.clone(), loc, true);
        self.cleanup_windows();
        self.windows.push(window.clone());
        if let Some(keyboard) = self.wayland.seat.get_keyboard() {
            keyboard.set_focus(
                self,
                Some(surface.wl_surface().clone()),
                SERIAL_COUNTER.next_serial(),
            );
        }
        // Send initial configure to new toplevel surface so client can map buffers
        surface.send_configure();
        self.windows.space.elements().for_each(|w| {
            if let Some(toplevel) = w.toplevel() {
                toplevel.send_pending_configure();
            }
        });
        self.update_active_window(Some(&window));
    }

    fn title_changed(&mut self, surface: ToplevelSurface) {
        crate::adapters::window::metadata::refresh(&Window::new_wayland_window(surface.clone()));
        if let Some(active) = self.active_window() {
            if active.toplevel() == Some(&surface) {
                self.update_active_window(Some(&active));
            }
        }
    }

    fn app_id_changed(&mut self, surface: ToplevelSurface) {
        crate::adapters::window::metadata::refresh(&Window::new_wayland_window(surface.clone()));
        if let Some(active) = self.active_window() {
            if active.toplevel() == Some(&surface) {
                self.update_active_window(Some(&active));
            }
        }
    }

    fn new_popup(&mut self, surface: PopupSurface, _positioner: PositionerState) {
        let _ = self.resources.popups.track_popup(PopupKind::Xdg(surface));
    }

    fn grab(&mut self, _surface: PopupSurface, _seat: wl_seat::WlSeat, _serial: Serial) {}

    fn move_request(&mut self, surface: ToplevelSurface, _seat: wl_seat::WlSeat, _serial: Serial) {
        if self.input.pointer_pressed_buttons.is_empty() && self.input.touch_slots.is_empty() {
            tracing::debug!("Ignoring xdg_toplevel.move_request: neither pointer button nor touch contact is held down");
            return;
        }

        let target_window = self.find_window_for_surface(&surface);

        if let Some(window) = target_window {
            if let Some(toplevel) = window.toplevel() {
                if toplevel.current_state().states.contains(
                    smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::State::Maximized,
                ) {
                    self.unmaximize_window(&window);
                }
            }

            if let Some(loc) = self.windows.space.element_location(&window) {
                self.windows.space.raise_element(&window, true);
                let drag_button = self
                    .input
                    .pointer_pressed_buttons
                    .iter()
                    .copied()
                    .next()
                    .unwrap_or(272);
                self.input.move_state = Some(crate::adapters::compositor::state::MoveState {
                    window,
                    initial_window_location: loc,
                    initial_pointer_location: self.input.pointer_location,
                    has_dragged: false,
                    button: drag_button,
                });
            }
        }
    }

    fn resize_request(
        &mut self,
        surface: ToplevelSurface,
        _seat: wl_seat::WlSeat,
        _serial: Serial,
        edges: smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::ResizeEdge,
    ) {
        if self.input.pointer_pressed_buttons.is_empty() && self.input.touch_slots.is_empty() {
            return;
        }

        let target_window = self.find_window_for_surface(&surface);
        if let Some(window) = target_window {
            if let Some(toplevel) = window.toplevel() {
                if toplevel.current_state().states.contains(
                    smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::State::Maximized,
                ) {
                    return;
                }
            }

            if let Some(loc) = self.windows.space.element_location(&window) {
                self.windows.space.raise_element(&window, true);
                let drag_button = self
                    .input
                    .pointer_pressed_buttons
                    .iter()
                    .copied()
                    .next()
                    .unwrap_or(272);
                let size = window.geometry().size;
                self.input.resize_state = Some(crate::adapters::compositor::state::ResizeState {
                    window,
                    initial_window_location: loc,
                    initial_window_size: size,
                    initial_pointer_location: self.input.pointer_location,
                    edges,
                    button: drag_button,
                });
            }
        }
    }

    fn maximize_request(&mut self, surface: ToplevelSurface) {
        if let Some(window) = self.find_window_for_surface(&surface) {
            self.maximize_window(&window);
        }
    }

    fn unmaximize_request(&mut self, surface: ToplevelSurface) {
        if let Some(window) = self.find_window_for_surface(&surface) {
            self.unmaximize_window(&window);
        }
    }

    fn minimize_request(&mut self, surface: ToplevelSurface) {
        if let Some(window) = self.find_window_for_surface(&surface) {
            self.minimize_window(&window);
        }
    }

    fn fullscreen_request(
        &mut self,
        surface: ToplevelSurface,
        output: Option<smithay::reexports::wayland_server::protocol::wl_output::WlOutput>,
    ) {
        if let Some(window) = self.find_window_for_surface(&surface) {
            self.fullscreen_window(&window, output);
        }
    }

    fn unfullscreen_request(&mut self, surface: ToplevelSurface) {
        if let Some(window) = self.find_window_for_surface(&surface) {
            self.unfullscreen_window(&window);
        }
    }

    fn reposition_request(
        &mut self,
        surface: PopupSurface,
        positioner: PositionerState,
        token: u32,
    ) {
        surface.with_pending_state(|state| {
            state.geometry = positioner.get_geometry();
            state.positioner = positioner;
        });
        surface.send_repositioned(token);
    }
}

pub(super) fn handle_commit(popups: &mut PopupManager, space: &Space<Window>, surface: &WlSurface) {
    if let Some(window) = space
        .elements()
        .find(|window| {
            window
                .toplevel()
                .map(|t| t.wl_surface() == surface)
                .unwrap_or(false)
        })
        .cloned()
    {
        // `CompositorHandler::commit` is the single owner of `Window::on_commit`.
        // Calling it here as well would process the same toplevel commit twice.
        crate::adapters::compositor::state::start_window_opening(&window);
        if let Some(toplevel) = window.toplevel() {
            let initial_configure_sent = with_states(surface, |states| {
                states
                    .data_map
                    .get::<XdgToplevelSurfaceData>()
                    .map(|d| match d.lock() {
                        Ok(data) => data.initial_configure_sent,
                        Err(poisoned) => {
                            tracing::warn!("recovering poisoned toplevel state lock");
                            poisoned.into_inner().initial_configure_sent
                        }
                    })
                    .unwrap_or(true)
            });
            if !initial_configure_sent {
                toplevel.send_configure();
            }
        }
    }

    popups.commit(surface);
    if let Some(PopupKind::Xdg(popup)) = popups.find_popup(surface) {
        if !popup.is_initial_configure_sent() {
            let _ = popup.send_configure();
        }
    }
}

delegate_xdg_shell!(BabydraState);
