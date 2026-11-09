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
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        let window = Window::new_wayland_window(surface);
        let count = self.space.elements().count() as i32;
        let offset = (count * 36) % 288;
        let loc = (48 + offset, 48 + offset);
        self.space.map_element(window.clone(), loc, true);
        self.seat.get_keyboard().unwrap().set_focus(
            self,
            Some(window.toplevel().unwrap().wl_surface().clone()),
            SERIAL_COUNTER.next_serial(),
        );
        self.space.elements().for_each(|w| {
            w.toplevel().unwrap().send_pending_configure();
        });
    }

    fn new_popup(&mut self, surface: PopupSurface, _positioner: PositionerState) {
        let _ = self.popups.track_popup(PopupKind::Xdg(surface));
    }

    fn grab(&mut self, _surface: PopupSurface, _seat: wl_seat::WlSeat, _serial: Serial) {}

    fn move_request(&mut self, surface: ToplevelSurface, _seat: wl_seat::WlSeat, _serial: Serial) {
        // Only allow moving if a pointer button (such as left click) is actively held down.
        // If the user already released the button (e.g. quick click or tap), ignore the request.
        if self.pointer_pressed_buttons.is_empty() {
            tracing::debug!("Ignoring xdg_toplevel.move_request: pointer button is not held down");
            return;
        }

        let target_window = self
            .space
            .elements()
            .find(|w| w.toplevel().map(|t| t == &surface).unwrap_or(false))
            .cloned();

        if let Some(window) = target_window {
            if let Some(loc) = self.space.element_location(&window) {
                self.space.raise_element(&window, true);
                let drag_button = self
                    .pointer_pressed_buttons
                    .iter()
                    .copied()
                    .next()
                    .unwrap_or(272);
                self.move_state = Some(crate::compositor::state::MoveState {
                    window,
                    initial_window_location: loc,
                    initial_pointer_location: self.pointer_location,
                    has_dragged: false,
                    button: drag_button,
                });
            }
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
        .find(|window| window.toplevel().unwrap().wl_surface() == surface)
        .cloned()
    {
        let initial_configure_sent = with_states(surface, |states| {
            states
                .data_map
                .get::<XdgToplevelSurfaceData>()
                .expect("xdg toplevel data missing")
                .lock()
                .expect("xdg toplevel state poisoned")
                .initial_configure_sent
        });
        if !initial_configure_sent {
            window.toplevel().unwrap().send_configure();
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
