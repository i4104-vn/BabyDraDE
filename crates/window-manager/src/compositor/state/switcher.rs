use std::time::Instant;

use smithay::{
    desktop::Window,
    reexports::wayland_protocols::xdg::shell::server::xdg_toplevel,
    utils::{Logical, Point, Size},
    wayland::compositor::with_states,
};

use super::{window_ops::window_state, BabydraState};

pub(crate) struct AltTabEntry {
    pub(crate) window: Window,
    pub(crate) app_name: String,
    pub(crate) title: String,
    pub(crate) original_location: Option<Point<i32, Logical>>,
    pub(crate) original_size: Option<Size<i32, Logical>>,
    pub(crate) was_maximized: bool,
    pub(crate) was_minimized: bool,
}

pub(crate) struct AltTabSwitcher {
    pub(crate) entries: Vec<AltTabEntry>,
    pub(crate) selected_index: usize,
    pub(crate) transition_started: Instant,
}

impl BabydraState {
    pub(crate) fn restore_alt_tab_windows(&mut self, switcher: &AltTabSwitcher) {
        for entry in &switcher.entries {
            if let Some(loc) = entry.original_location {
                if !entry.was_minimized {
                    self.space.map_element(entry.window.clone(), loc, false);
                }
            }
            if entry.was_maximized {
                if let Some(toplevel) = entry.window.toplevel() {
                    toplevel.with_pending_state(|state| {
                        state.states.set(xdg_toplevel::State::Maximized);
                        state.size = entry.original_size;
                    });
                    toplevel.send_configure();
                }
            } else if let Some(orig_size) = entry.original_size {
                if let Some(toplevel) = entry.window.toplevel() {
                    toplevel.with_pending_state(|state| {
                        state.states.unset(xdg_toplevel::State::Maximized);
                        state.size = Some(orig_size);
                    });
                    toplevel.send_configure();
                }
            }
        }
        self.space.refresh();
    }

    pub(crate) fn cycle_alt_tab(&mut self, backwards: bool) {
        if let Some(mut switcher) = self.alt_tab_switcher.take() {
            let selected_window = switcher
                .entries
                .get(switcher.selected_index)
                .map(|entry| entry.window.clone());
            self.cleanup_windows();
            let all_windows = self.windows.clone();
            switcher
                .entries
                .retain(|entry| all_windows.iter().any(|window| window == &entry.window));

            if switcher.entries.is_empty() {
                return;
            }

            if switcher.entries.len() == 1 {
                switcher.selected_index = 0;
                switcher.transition_started = Instant::now();
                self.alt_tab_switcher = Some(switcher);
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

        self.cleanup_windows();
        let entries = self
            .windows
            .iter()
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

                let original_location = self.space.element_location(window);
                let original_size = Some(window.geometry().size);
                let was_maximized = window
                    .toplevel()
                    .map(|t| t.current_state().states.contains(xdg_toplevel::State::Maximized))
                    .unwrap_or(false);
                let was_minimized = window_state(window).lock().unwrap().is_minimized;

                Some(AltTabEntry {
                    window: window.clone(),
                    app_name,
                    title,
                    original_location,
                    original_size,
                    was_maximized,
                    was_minimized,
                })
            })
            .collect::<Vec<_>>();

        if entries.is_empty() {
            return;
        }

        let selected_index = if entries.len() == 1 {
            0
        } else if backwards {
            entries.len() - 1
        } else {
            1
        };
        self.alt_tab_switcher = Some(AltTabSwitcher {
            entries,
            selected_index,
            transition_started: Instant::now(),
        });
    }
}
