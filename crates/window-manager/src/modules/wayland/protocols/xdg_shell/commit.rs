use smithay::{
    desktop::{PopupKind, PopupManager, Space, Window},
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    wayland::{compositor::with_states, shell::xdg::XdgToplevelSurfaceData},
};

/// Processes a commit on a toplevel or popup surface.
pub(crate) fn handle_commit(popups: &mut PopupManager, space: &Space<Window>, surface: &WlSurface) {
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
        crate::app::state::start_window_opening(&window);
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
