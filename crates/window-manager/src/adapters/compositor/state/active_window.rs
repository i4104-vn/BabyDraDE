use smithay::{desktop::Window, reexports::wayland_server::Resource};

use super::BabydraState;

impl BabydraState {
    pub(crate) fn update_active_window(&self, window: Option<&Window>) {
        let info = if let Some(window) = window {
            if let Some(toplevel) = window.toplevel() {
                let metadata =
                    crate::adapters::compositor::state::metadata::get(window).unwrap_or_default();
                let process_id = self
                    .display_handle
                    .get_client(toplevel.wl_surface().id())
                    .ok()
                    .and_then(|client| client.get_credentials(&self.display_handle).ok())
                    .and_then(|credentials| u32::try_from(credentials.pid).ok());
                services::system::window::ActiveWindowInfo {
                    app_id: metadata.app_id,
                    title: metadata.title,
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
