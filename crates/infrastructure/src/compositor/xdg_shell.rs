use application::WindowCommand;
use domain::{Size, WindowId};

use super::state::BabyDraCompositorState;

/// Manages XDG Shell toplevel windows created by desktop applications.
pub struct XdgShellHandler;

impl XdgShellHandler {
    /// Handler when an external Wayland client creates a new toplevel window.
    pub fn handle_new_toplevel(
        state: &mut BabyDraCompositorState,
        title: String,
        app_id: Option<String>,
        initial_size: Option<Size>,
    ) -> WindowId {
        let id = state.allocate_window_id();
        let size = initial_size.unwrap_or(Size::new(800, 600));

        tracing::info!(
            "New toplevel window: id={}, title='{}', app_id={:?}",
            id,
            title,
            app_id
        );

        let cmd = WindowCommand::OpenWindow {
            id,
            title,
            app_id,
            initial_size: size,
        };
        state.handle_command(cmd.into());

        id
    }

    /// Handler when an external client closes / destroys a toplevel window.
    pub fn handle_toplevel_destroyed(state: &mut BabyDraCompositorState, id: WindowId) {
        tracing::info!("Toplevel destroyed: id={}", id);
        let cmd = WindowCommand::CloseWindow { id };
        state.handle_command(cmd.into());
    }

    /// Handler when client requests maximize.
    pub fn handle_request_maximize(state: &mut BabyDraCompositorState, id: WindowId) {
        let cmd = WindowCommand::ToggleMaximize { id };
        state.handle_command(cmd.into());
    }

    /// Handler when client requests minimize.
    pub fn handle_request_minimize(state: &mut BabyDraCompositorState, id: WindowId) {
        let cmd = WindowCommand::MinimizeWindow { id };
        state.handle_command(cmd.into());
    }
}
