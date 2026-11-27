use std::time::{Duration, Instant};

use animation::{Easing, Tween, SWITCHER_PREVIEW_DURATION, SWITCHER_SCROLL_DURATION};
use smithay::reexports::wayland_server::Resource;

use crate::app::state::BabydraState;
use crate::modules::shell::switcher::models::{step_target, AltTabEntry, AltTabSwitcher};

impl BabydraState {
    pub(crate) fn cycle_alt_tab(&mut self, backwards: bool) {
        if self
            .overlays
            .alt_tab_switcher
            .as_ref()
            .is_some_and(|switcher| switcher.closing)
        {
            return;
        }
        if let Some(mut switcher) = self.overlays.alt_tab_switcher.take() {
            switcher.hovered = None;
            let entries_before = switcher.entries.len();
            let selected_window = switcher
                .entries
                .get(switcher.selected_index)
                .map(|entry| entry.window.clone());
            self.cleanup_windows();
            let all_windows = self.windows.clone();
            switcher
                .entries
                .retain(|entry| all_windows.iter().any(|window| window == &entry.window));
            let entries_changed = switcher.entries.len() != entries_before;

            if switcher.entries.is_empty() {
                return;
            }

            let direction = if backwards { -1.0 } else { 1.0 };

            if switcher.entries.len() == 1 {
                switcher.selected_index = 0;
                switcher.scroll.snap_to(0.0);
                switcher.highlight.snap_to(0.0);
                switcher.scroll_direction = direction;
                switcher.restart_stage();
                self.overlays.alt_tab_switcher = Some(switcher);
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
            let next = selected_index as i64 + if backwards { -1 } else { 1 };
            switcher.selected_index = next.rem_euclid(switcher.entries.len() as i64) as usize;

            if entries_changed {
                // The entry list changed (a window opened or closed): snap the
                // scroll position so it cannot land on an item that is gone.
                switcher.scroll.snap_to(switcher.selected_index as f64);
                switcher.highlight.snap_to(switcher.selected_index as f64);
            } else {
                // Accumulate onto the target instead of the current position,
                // so repeated Tab presses do not jump back to the previous step.
                let (target, wrapped) =
                    step_target(switcher.scroll.target(), direction, switcher.entries.len());
                // While Tab is held the backend sends repeated presses. Use a
                // short linear step so the dock glides continuously instead of
                // restarting a long easing curve and jumping at the end.
                let rapid = switcher
                    .last_step
                    .is_some_and(|last| last.elapsed() < Duration::from_millis(150));
                switcher.last_step = Some(Instant::now());
                if wrapped {
                    switcher.scroll.snap_to(target);
                    switcher.highlight.snap_to(target);
                } else {
                    let (duration, easing) = if rapid {
                        (
                            Duration::from_millis(common::constants::KEY_REPEAT_INTERVAL_MS),
                            Easing::Linear,
                        )
                    } else {
                        (SWITCHER_SCROLL_DURATION, Easing::EaseOutCubic)
                    };
                    switcher.scroll.retarget(target, duration, easing);
                    switcher.highlight.retarget(target, duration, easing);
                }
            }
            switcher.scroll_direction = direction;
            switcher.restart_stage();
            self.overlays.alt_tab_switcher = Some(switcher);
            return;
        }

        // The launcher is an overlay too; it must not sit above the switcher.
        if self.overlays.launcher.is_visible() {
            self.overlays.launcher.toggle();
        }

        self.cleanup_windows();
        let entries = self
            .windows
            .iter()
            .rev()
            .filter(|w| !crate::modules::window::ops::is_window_exiting(w))
            .filter_map(|window| {
                let metadata = crate::modules::window::metadata::get(window)?;
                // Use the window's real client pid so the switcher label and icon
                // come from the same installed entry as the topbar.
                let process_id = window
                    .toplevel()
                    .and_then(|toplevel| {
                        self.wayland
                            .display_handle
                            .get_client(toplevel.wl_surface().id())
                            .ok()
                    })
                    .and_then(|client| client.get_credentials(&self.wayland.display_handle).ok())
                    .and_then(|credentials| u32::try_from(credentials.pid).ok())
                    .filter(|pid| *pid > 0);
                let icon_app_id = services::system::window::get_window_icon_id(
                    &metadata.app_id,
                    &metadata.title,
                    process_id,
                );
                let app_name = services::system::window::get_app_display_name_with_pid(
                    &metadata.app_id,
                    &metadata.title,
                    process_id,
                );
                let app_name = if app_name.is_empty() {
                    "Application".to_string()
                } else {
                    app_name
                };

                // Resolve the icon eagerly here (on the event-loop thread, before any
                // rendering happens) so the render path never has to call into GTK/GIO.
                // Wrapped in catch_unwind so a GTK/GIO panic inside icon resolution
                // cannot take down the whole compositor — the switcher simply falls
                // back to a coloured badge when the icon is unavailable.
                let icon_pixbuf =
                    std::panic::catch_unwind(|| ui_kit::get_app_icon_pixbuf_any(&icon_app_id, 32))
                        .unwrap_or(None);

                Some(AltTabEntry {
                    window: window.clone(),
                    app_name,
                    icon_pixbuf,
                })
            })
            .collect::<Vec<_>>();

        if entries.is_empty() {
            return;
        }

        let selected_index = match (entries.len() == 1, backwards) {
            (true, _) => 0,
            (false, true) => entries.len() - 1,
            (false, false) => 1,
        };
        let selected = selected_index.min(entries.len() - 1);
        self.overlays.alt_tab_switcher = Some(AltTabSwitcher {
            entries,
            selected_index: selected,
            hovered: None,
            scroll: Tween::settled(selected as f64),
            highlight: Tween::settled(selected as f64),
            stage: Tween::new(0.0, 1.0, SWITCHER_PREVIEW_DURATION, Easing::EaseOutCubic),
            presence: Tween::new(0.0, 1.0, SWITCHER_PREVIEW_DURATION, Easing::EaseOutCubic),
            closing: false,
            last_step: None,
            held_direction: None,
            repeat_started: None,
            last_repeat: None,
            scroll_direction: if backwards { -1.0 } else { 1.0 },
        });
    }
}
