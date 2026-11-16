//! Forces applications to use BabyDraDE's compositor-owned window frame.

use smithay::{
    delegate_xdg_decoration,
    reexports::wayland_protocols::xdg::decoration::zv1::server::zxdg_toplevel_decoration_v1::Mode,
    wayland::shell::xdg::{decoration::XdgDecorationHandler, ToplevelSurface},
};

use super::super::state::BabydraState;

impl XdgDecorationHandler for BabydraState {
    fn new_decoration(&mut self, toplevel: ToplevelSurface) {
        toplevel.with_pending_state(|state| state.decoration_mode = Some(Mode::ServerSide));
        toplevel.send_configure();
    }

    fn request_mode(&mut self, toplevel: ToplevelSurface, _mode: Mode) {
        toplevel.with_pending_state(|state| state.decoration_mode = Some(Mode::ServerSide));
        toplevel.send_configure();
    }

    fn unset_mode(&mut self, toplevel: ToplevelSurface) {
        toplevel.with_pending_state(|state| state.decoration_mode = Some(Mode::ServerSide));
        toplevel.send_configure();
    }
}

delegate_xdg_decoration!(BabydraState);
