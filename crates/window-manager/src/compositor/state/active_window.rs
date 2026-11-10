use smithay::{
    desktop::Window, reexports::wayland_server::Resource, wayland::compositor::with_states,
};

use super::BabydraState;

impl BabydraState {
    pub(crate) fn update_active_window(&self, window: Option<&Window>) {
        let info = if let Some(window) = window {
            if let Some(toplevel) = window.toplevel() {
                let (app_id, title) = with_states(toplevel.wl_surface(), |states| {
                    states
                        .data_map
                        .get::<smithay::wayland::shell::xdg::XdgToplevelSurfaceData>()
                        .map(|data| {
                            let guard = data.lock().unwrap();
                            (
                                guard.app_id.clone().unwrap_or_default(),
                                guard.title.clone().unwrap_or_default(),
                            )
                        })
                })
                .unwrap_or_default();
                let process_id = self
                    .display_handle
                    .get_client(toplevel.wl_surface().id())
                    .ok()
                    .and_then(|client| client.get_credentials(&self.display_handle).ok())
                    .and_then(|credentials| u32::try_from(credentials.pid).ok());
                services::system::window::ActiveWindowInfo {
                    app_id,
                    title,
                    process_id,
                }
            } else {
                services::system::window::ActiveWindowInfo::default()
            }
        } else {
            services::system::window::ActiveWindowInfo::default()
        };
        services::system::window::save_active_window(&info);
    }
}
