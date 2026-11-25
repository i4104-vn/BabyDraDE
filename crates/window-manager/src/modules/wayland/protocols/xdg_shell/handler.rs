use smithay::{
    delegate_xdg_shell,
    desktop::{PopupKind, Window},
    reexports::wayland_server::protocol::wl_seat,
    utils::{Serial, SERIAL_COUNTER},
    wayland::shell::xdg::{
        PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState,
    },
};

use crate::app::state::BabydraState;

use super::window_ops;

impl XdgShellHandler for BabydraState {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.wayland.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        let window = Window::new_wayland_window(surface.clone());
        crate::modules::window::metadata::refresh(&window);
        let count = self.windows.space.elements().count() as i32;
        let offset = (count * 36) % 288;
        let loc = (48 + offset, 48 + offset);
        self.windows.space.map_element(window.clone(), loc, true);
        self.cleanup_windows();
        self.windows.push(window.clone());
        if let Some(keyboard) = self.wayland.seat().get_keyboard() {
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
        crate::modules::window::metadata::refresh(&Window::new_wayland_window(surface.clone()));
        if let Some(active) = self.active_window() {
            if active.toplevel() == Some(&surface) {
                self.update_active_window(Some(&active));
            }
        }
    }

    fn app_id_changed(&mut self, surface: ToplevelSurface) {
        crate::modules::window::metadata::refresh(&Window::new_wayland_window(surface.clone()));
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
        window_ops::move_request(self, surface);
    }

    fn resize_request(
        &mut self,
        surface: ToplevelSurface,
        _seat: wl_seat::WlSeat,
        _serial: Serial,
        edges: smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::ResizeEdge,
    ) {
        window_ops::resize_request(self, surface, edges);
    }

    fn maximize_request(&mut self, surface: ToplevelSurface) {
        window_ops::maximize_request(self, surface);
    }

    fn unmaximize_request(&mut self, surface: ToplevelSurface) {
        window_ops::unmaximize_request(self, surface);
    }

    fn minimize_request(&mut self, surface: ToplevelSurface) {
        window_ops::minimize_request(self, surface);
    }

    fn fullscreen_request(
        &mut self,
        surface: ToplevelSurface,
        output: Option<smithay::reexports::wayland_server::protocol::wl_output::WlOutput>,
    ) {
        window_ops::fullscreen_request(self, surface, output);
    }

    fn unfullscreen_request(&mut self, surface: ToplevelSurface) {
        window_ops::unfullscreen_request(self, surface);
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

delegate_xdg_shell!(BabydraState);
