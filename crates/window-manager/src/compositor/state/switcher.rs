use std::time::Instant;

use smithay::{desktop::Window, wayland::compositor::with_states};

use super::BabydraState;

pub(crate) struct AltTabEntry {
    pub(crate) window: Window,
    pub(crate) app_name: String,
    pub(crate) title: String,
}

pub(crate) struct AltTabSwitcher {
    pub(crate) entries: Vec<AltTabEntry>,
    pub(crate) selected_index: usize,
    pub(crate) transition_started: Instant,
}

impl BabydraState {
    pub(crate) fn cycle_alt_tab(&mut self, backwards: bool) {
        if let Some(mut switcher) = self.alt_tab_switcher.take() {
            let selected_window = switcher
                .entries
                .get(switcher.selected_index)
                .map(|entry| entry.window.clone());
            let mapped_windows = self.space.elements().cloned().collect::<Vec<_>>();
            switcher
                .entries
                .retain(|entry| mapped_windows.iter().any(|window| window == &entry.window));

            if switcher.entries.len() < 2 {
                return;
            }

            let selected_index = selected_window
                .and_then(|selected| {
                    switcher
                        .entries
                        .iter()
                        .position(|entry| entry.window == selected)
                })
                .unwrap_or(0);
            switcher.selected_index = if backwards {
                (selected_index + switcher.entries.len() - 1) % switcher.entries.len()
            } else {
                (selected_index + 1) % switcher.entries.len()
            };
            switcher.transition_started = Instant::now();
            self.alt_tab_switcher = Some(switcher);
            return;
        }

        let entries = self
            .space
            .elements()
            .rev()
            .filter_map(|window| {
                let toplevel = window.toplevel()?;
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
                let app_name = services::system::window::get_app_display_name(&app_id, &title);
                let app_name = if app_name.is_empty() {
                    "Application".to_string()
                } else {
                    app_name
                };
                let title = if title.trim().is_empty() {
                    app_name.clone()
                } else {
                    title
                };

                Some(AltTabEntry {
                    window: window.clone(),
                    app_name,
                    title,
                })
            })
            .collect::<Vec<_>>();

        if entries.len() < 2 {
            return;
        }

        let selected_index = if backwards { entries.len() - 1 } else { 1 };
        self.alt_tab_switcher = Some(AltTabSwitcher {
            entries,
            selected_index,
            transition_started: Instant::now(),
        });
    }
}
