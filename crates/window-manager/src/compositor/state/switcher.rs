use std::time::Duration;

use animation::{Easing, Tween, DEFAULT_DURATION};
use gtk4::gdk_pixbuf::Pixbuf;
use smithay::{
    desktop::Window,
    reexports::wayland_protocols::xdg::shell::server::xdg_toplevel,
    utils::{Logical, Point, Size},
    wayland::compositor::with_states,
};

use super::{window_state, BabydraState};
use common::constants::{DOCK_MAX_VISIBLE_ITEMS, DOCK_TARGET_ANIM_MS, STAGE_ANIM_MS};

/// One entry in the Alt+Tab switcher list.
/// `icon_pixbuf` is resolved eagerly when the switcher opens so that
/// the render path never has to call into GTK/GIO.
pub(crate) struct AltTabEntry {
    pub(crate) window: Window,
    /// Wayland app_id (e.g. "org.mozilla.firefox"). Kept for debugging and
    /// potential future use; icon resolution happens at construction time.
    #[allow(dead_code)]
    pub(crate) app_id: String,
    pub(crate) app_name: String,
    /// Pre-loaded icon, or `None` if the app has no resolvable icon.
    pub(crate) icon_pixbuf: Option<Pixbuf>,
    pub(crate) original_location: Option<Point<i32, Logical>>,
    pub(crate) original_size: Option<Size<i32, Logical>>,
    pub(crate) was_maximized: bool,
    pub(crate) was_minimized: bool,
}

pub(crate) struct AltTabSwitcher {
    pub(crate) entries: Vec<AltTabEntry>,
    pub(crate) selected_index: usize,
    /// Continuous carousel position (fractional slots) of the right-side dock.
    /// Driven by `Tween` so repeated Tab presses accumulate smoothly.
    pub(crate) scroll: Tween,
    /// 0..1 fade of the small preview beside the active icon.
    pub(crate) stage: Tween,
    /// +1.0 when moving forward, -1.0 when going backwards with Shift+Tab.
    pub(crate) scroll_direction: f64,
}

impl AltTabSwitcher {
    /// Continuous (fractional) selection position for the right-side dock.
    pub(crate) fn visual_selection(&self) -> f64 {
        self.scroll.value()
    }

    /// 0..1 progress of the small preview fade.
    pub(crate) fn stage_progress(&self) -> f64 {
        self.stage.value()
    }

    /// Restarts the preview fade from 0 -> 1 after a selection change.
    fn restart_stage(&mut self) {
        self.stage.restart(
            0.0,
            1.0,
            Duration::from_millis(STAGE_ANIM_MS),
            Easing::Linear,
        );
    }
}

impl BabydraState {
    pub(crate) fn restore_alt_tab_windows(&mut self, switcher: &AltTabSwitcher) {
        for entry in &switcher.entries {
            if let Some(loc) = entry.original_location {
                if !entry.was_minimized {
                    self.space.map_element(entry.window.clone(), loc, false);
                }
            }
            match (entry.was_maximized, entry.original_size) {
                (true, _) => {
                    if let Some(toplevel) = entry.window.toplevel() {
                        toplevel.with_pending_state(|state| {
                            state.states.set(xdg_toplevel::State::Maximized);
                            state.size = entry.original_size;
                        });
                        toplevel.send_configure();
                    }
                }
                (false, Some(orig_size)) => {
                    if let Some(toplevel) = entry.window.toplevel() {
                        toplevel.with_pending_state(|state| {
                            state.states.unset(xdg_toplevel::State::Maximized);
                            state.size = Some(orig_size);
                        });
                        toplevel.send_configure();
                    }
                }
                (false, None) => {}
            }
        }
        self.space.refresh();
    }

    pub(crate) fn cycle_alt_tab(&mut self, backwards: bool) {
        if let Some(mut switcher) = self.alt_tab_switcher.take() {
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
                switcher.scroll_direction = direction;
                switcher.restart_stage();
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

            if entries_changed {
                // Danh sách entry đã đổi (có cửa sổ đóng/mở): căn lại để
                // tránh cuộn tới một item không còn tồn tại.
                switcher.scroll.snap_to(switcher.selected_index as f64);
            } else {
                // Cộng dồn vào đích thay vì vị trí hiện tại, nhờ vậy bấm Tab
                // liên tiếp không bị nhảy lùi về đầu bước trước.
                let few_items = switcher.entries.len() < DOCK_MAX_VISIBLE_ITEMS;
                // Với ít app, các item đứng yên và chỉ khung active trượt giữa
                // chúng. Không cộng dồn như carousel (sẽ trượt ra khỏi dock sau
                // vài vòng lặp) mà đi thẳng tới slot của item được chọn.
                let target = if few_items {
                    switcher.selected_index as f64
                } else {
                    switcher.scroll.target() + direction
                };
                let (duration, easing) = if few_items {
                    (
                        Duration::from_millis(DOCK_TARGET_ANIM_MS),
                        Easing::EaseInOutSine,
                    )
                } else {
                    (DEFAULT_DURATION, Easing::EaseInOutCubic)
                };
                switcher.scroll.retarget(target, duration, easing);
            }
            switcher.scroll_direction = direction;
            switcher.restart_stage();
            self.alt_tab_switcher = Some(switcher);
            return;
        }

        // The launcher is an overlay too; it must not sit above the switcher.
        if self.launcher.is_visible() {
            self.launcher.toggle();
        }

        self.cleanup_windows();
        let entries = self
            .windows
            .iter()
            .rev()
            .filter(|w| !super::is_window_exiting(w))
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

                // Resolve the icon eagerly here (on the event-loop thread, before any
                // rendering happens) so the render path never has to call into GTK/GIO.
                // Wrapped in catch_unwind so a GTK/GIO panic inside icon resolution
                // cannot take down the whole compositor — the switcher simply falls
                // back to a coloured badge when the icon is unavailable.
                let icon_pixbuf =
                    std::panic::catch_unwind(|| ui_kit::get_app_icon_pixbuf_any(&app_id, 32))
                        .unwrap_or(None);

                let original_location = self.space.element_location(window);
                let original_size = Some(window.geometry().size);
                let was_maximized = window
                    .toplevel()
                    .map(|t| {
                        t.current_state()
                            .states
                            .contains(xdg_toplevel::State::Maximized)
                    })
                    .unwrap_or(false);
                let was_minimized = window_state(window).lock().unwrap().is_minimized;

                Some(AltTabEntry {
                    window: window.clone(),
                    app_id,
                    app_name,
                    icon_pixbuf,
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

        let selected_index = match (entries.len() == 1, backwards) {
            (true, _) => 0,
            (false, true) => entries.len() - 1,
            (false, false) => 1,
        };
        self.alt_tab_switcher = Some(AltTabSwitcher {
            entries,
            selected_index,
            scroll: Tween::settled(selected_index as f64),
            stage: Tween::new(
                0.0,
                1.0,
                Duration::from_millis(STAGE_ANIM_MS),
                Easing::Linear,
            ),
            scroll_direction: if backwards { -1.0 } else { 1.0 },
        });
    }
}
