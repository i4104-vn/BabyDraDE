use std::time::{Duration, Instant};

use animation::{Easing, Tween};
use gtk4::gdk_pixbuf::Pixbuf;
use smithay::{desktop::Window, wayland::compositor::with_states};

use super::BabydraState;
use common::constants::{DOCK_MAX_VISIBLE_ITEMS, SWITCHER_PREVIEW_MS, SWITCHER_SCROLL_MS};

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
}

pub(crate) struct AltTabSwitcher {
    pub(crate) entries: Vec<AltTabEntry>,
    pub(crate) selected_index: usize,
    /// Continuous carousel position (fractional slots) of the right-side dock.
    /// Driven by `Tween` so repeated Tab presses accumulate smoothly.
    pub(crate) scroll: Tween,
    /// 0..1 fade of the small preview beside the active icon.
    pub(crate) stage: Tween,
    /// 0..1 presence driving the backdrop fade and slide in/out.
    pub(crate) presence: Tween,
    /// Rendered while the close animation plays before the state is removed.
    pub(crate) closing: bool,
    /// Last Tab step, used to keep held-key repeats continuous.
    pub(crate) last_step: Option<Instant>,
    /// Held Tab direction and timers for backend-independent auto-repeat.
    pub(crate) held_direction: Option<i32>,
    pub(crate) repeat_started: Option<Instant>,
    pub(crate) last_repeat: Option<Instant>,
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
            Duration::from_millis(SWITCHER_PREVIEW_MS),
            Easing::EaseOutCubic,
        );
    }

    pub(crate) fn presence(&self) -> f64 {
        self.presence.value()
    }

    /// First Tab press starts repeat tracking; backend repeats are ignored
    /// because `tick_key_repeat` drives held navigation.
    pub(crate) fn tab_pressed(&mut self, direction: i32) -> bool {
        if self.closing || self.held_direction == Some(direction) {
            return false;
        }
        self.held_direction = Some(direction);
        let now = Instant::now();
        self.repeat_started = Some(now);
        self.last_repeat = Some(now);
        true
    }

    pub(crate) fn tab_released(&mut self, direction: i32) {
        if self.held_direction == Some(direction) {
            self.stop_key_repeat();
        }
    }

    fn stop_key_repeat(&mut self) {
        self.held_direction = None;
        self.repeat_started = None;
        self.last_repeat = None;
    }

    pub(crate) fn tick_key_repeat(&mut self) {
        let Some(direction) = self.held_direction else {
            return;
        };
        let Some(started) = self.repeat_started else {
            return;
        };
        let now = Instant::now();
        let delay = Duration::from_millis(common::constants::KEY_REPEAT_DELAY_MS);
        let interval = Duration::from_millis(common::constants::KEY_REPEAT_INTERVAL_MS);
        if now.duration_since(started) >= delay
            && now.duration_since(self.last_repeat.unwrap_or(started)) >= interval
        {
            self.last_repeat = Some(now);
            self.repeat_step(direction);
        }
    }

    fn repeat_step(&mut self, direction: i32) {
        let count = self.entries.len();
        if count < 2 {
            return;
        }
        let target = if count < DOCK_MAX_VISIBLE_ITEMS {
            let current = self.selected_index as i64;
            (current + direction as i64).rem_euclid(count as i64) as f64
        } else {
            (self.scroll.target() + direction as f64).clamp(0.0, count as f64 - 1.0)
        };
        self.selected_index = (target.round() as usize).min(count - 1);
        self.scroll.retarget(
            target,
            Duration::from_millis(common::constants::KEY_REPEAT_INTERVAL_MS),
            Easing::Linear,
        );
        self.last_step = Some(Instant::now());
    }

    pub(crate) fn begin_close(&mut self) {
        if self.closing {
            return;
        }
        self.closing = true;
        self.stop_key_repeat();
        self.presence.retarget(
            0.0,
            Duration::from_millis(SWITCHER_PREVIEW_MS),
            Easing::EaseInOutCubic,
        );
    }
}

impl BabydraState {
    pub(crate) fn cycle_alt_tab(&mut self, backwards: bool) {
        if self
            .alt_tab_switcher
            .as_ref()
            .is_some_and(|switcher| switcher.closing)
        {
            return;
        }
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
                // While Tab is held the backend sends repeated presses. Use a
                // short linear step so the dock glides continuously instead of
                // restarting a long easing curve and jumping at the end.
                let rapid = switcher
                    .last_step
                    .is_some_and(|last| last.elapsed() < Duration::from_millis(150));
                switcher.last_step = Some(Instant::now());
                let (duration, easing) = if rapid {
                    (
                        Duration::from_millis(common::constants::KEY_REPEAT_INTERVAL_MS),
                        Easing::Linear,
                    )
                } else {
                    (
                        Duration::from_millis(SWITCHER_SCROLL_MS),
                        Easing::EaseOutCubic,
                    )
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

                Some(AltTabEntry {
                    window: window.clone(),
                    app_id,
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
        self.alt_tab_switcher = Some(AltTabSwitcher {
            entries,
            selected_index,
            scroll: Tween::settled(selected_index as f64),
            stage: Tween::new(
                0.0,
                1.0,
                Duration::from_millis(SWITCHER_PREVIEW_MS),
                Easing::EaseOutCubic,
            ),
            presence: Tween::new(
                0.0,
                1.0,
                Duration::from_millis(SWITCHER_PREVIEW_MS),
                Easing::EaseOutCubic,
            ),
            closing: false,
            last_step: None,
            held_direction: None,
            repeat_started: None,
            last_repeat: None,
            scroll_direction: if backwards { -1.0 } else { 1.0 },
        });
    }
}
