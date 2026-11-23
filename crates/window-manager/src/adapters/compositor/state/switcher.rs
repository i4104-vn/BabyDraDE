use std::time::{Duration, Instant};

use animation::{
    Easing, Tween, HOVER_DURATION, SWITCHER_PREVIEW_DURATION, SWITCHER_SCROLL_DURATION,
};
use gtk4::gdk_pixbuf::Pixbuf;
use smithay::{desktop::Window, utils::SERIAL_COUNTER};

use super::BabydraState;

/// One entry in the Alt+Tab switcher list.
/// `icon_pixbuf` is resolved eagerly when the switcher opens so that
/// the render path never has to call into GTK/GIO.
pub(crate) struct AltTabEntry {
    pub(crate) window: Window,
    pub(crate) app_name: String,
    /// Pre-loaded icon, or `None` if the app has no resolvable icon.
    pub(crate) icon_pixbuf: Option<Pixbuf>,
}

pub(crate) struct AltTabSwitcher {
    pub(crate) entries: Vec<AltTabEntry>,
    pub(crate) selected_index: usize,
    /// Icon under the pointer; overrides `selected_index` while hovering.
    pub(crate) hovered: Option<usize>,
    /// Continuous carousel position (fractional slots) of the right-side dock.
    /// Driven by `Tween` so repeated Tab presses accumulate smoothly.
    pub(crate) scroll: Tween,
    /// Animated highlight position, independent of the scrolling content.
    pub(crate) highlight: Tween,
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
        self.stage
            .restart(0.0, 1.0, SWITCHER_PREVIEW_DURATION, Easing::EaseOutCubic);
    }

    pub(crate) fn presence(&self) -> f64 {
        self.presence.value()
    }

    pub(crate) fn set_hover(&mut self, index: Option<usize>) {
        if self.hovered != index {
            self.hovered = index;
            if let Some(index) = index {
                self.highlight
                    .retarget(index as f64, HOVER_DURATION, Easing::EaseOutCubic);
            }
        }
    }

    pub(crate) fn highlight_position(&self) -> f64 {
        self.highlight.value()
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
        let (target, wrapped) = step_target(self.scroll.target(), direction as f64, count);
        self.selected_index = target.round() as usize;
        let duration = Duration::from_millis(common::constants::KEY_REPEAT_INTERVAL_MS);
        if wrapped {
            self.scroll.snap_to(target);
            self.highlight.snap_to(target);
        } else {
            self.scroll.retarget(target, duration, Easing::Linear);
            self.highlight.retarget(target, duration, Easing::Linear);
        }
        self.last_step = Some(Instant::now());
    }

    pub(crate) fn begin_close(&mut self) {
        if self.closing {
            return;
        }
        self.closing = true;
        self.hovered = None;
        self.stop_key_repeat();
        self.presence
            .retarget(0.0, SWITCHER_PREVIEW_DURATION, Easing::EaseInOutCubic);
    }

    pub(crate) fn drag_by(&mut self, slots: f64) {
        let count = self.entries.len();
        if count < 2 || self.closing {
            return;
        }
        self.hovered = None;
        let current = self.scroll.target();
        let (next, _) = step_target(current, slots, count);
        self.selected_index = next.round() as usize;
        self.scroll.snap_to(next);
        self.highlight.snap_to(next);
    }

    pub(crate) fn finish_drag(&mut self) {
        let count = self.entries.len();
        if count < 2 || self.closing {
            return;
        }
        let nearest = self.scroll.value().round().rem_euclid(count as f64);
        self.selected_index = nearest as usize;
        self.scroll
            .retarget(nearest, SWITCHER_SCROLL_DURATION, Easing::EaseOutCubic);
        self.highlight
            .retarget(nearest, SWITCHER_SCROLL_DURATION, Easing::EaseOutCubic);
        self.restart_stage();
    }
}

/// Next dock position plus whether it wrapped past an end. Wrapping snaps
/// straight to the first/last icon so there is no empty scroll or drift.
fn step_target(current: f64, direction: f64, count: usize) -> (f64, bool) {
    let raw = current + direction;
    let n = count as f64;
    if raw >= n {
        (0.0, true)
    } else if raw < 0.0 {
        (n - 1.0, true)
    } else {
        (raw, false)
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
            let next = selected_index as i64 + if backwards { -1 } else { 1 };
            switcher.selected_index = next.rem_euclid(switcher.entries.len() as i64) as usize;

            if entries_changed {
                // Danh sách entry đã đổi (có cửa sổ đóng/mở): căn lại để
                // tránh cuộn tới một item không còn tồn tại.
                switcher.scroll.snap_to(switcher.selected_index as f64);
                switcher.highlight.snap_to(switcher.selected_index as f64);
            } else {
                // Cộng dồn vào đích thay vì vị trí hiện tại, nhờ vậy bấm Tab
                // liên tiếp không bị nhảy lùi về đầu bước trước.
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
                let metadata = crate::adapters::window::metadata::get(window)?;
                let app_id = metadata.app_id;
                let app_name = if metadata.app_name.is_empty() {
                    "Application".to_string()
                } else {
                    metadata.app_name
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
        self.alt_tab_switcher = Some(AltTabSwitcher {
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

    /// Opens the switcher for a finger drag and returns the index to scrub from.
    pub(crate) fn begin_switcher_scrub(&mut self) -> Option<usize> {
        if self.alt_tab_switcher.is_none() {
            self.cycle_alt_tab(false);
        }
        self.alt_tab_switcher
            .as_ref()
            .map(|switcher| switcher.selected_index)
    }

    /// Focuses the current switcher entry and starts the close animation.
    pub(crate) fn commit_switcher_selection(&mut self) {
        let window = self.alt_tab_switcher.as_ref().and_then(|switcher| {
            switcher
                .entries
                .get(switcher.selected_index)
                .map(|entry| entry.window.clone())
        });
        if let Some(window) = window {
            self.unminimize_window(&window);
            crate::adapters::input::keyboard::focus::focus_window(
                self,
                window,
                SERIAL_COUNTER.next_serial(),
            );
        }
        if let Some(switcher) = self.alt_tab_switcher.as_mut() {
            switcher.begin_close();
        }
    }

    /// Moves the active switcher entry with the finger. `steps` is the number
    /// of icons travelled since the drag started (may be negative).
    pub(crate) fn scrub_switcher(&mut self, start_x: f64, base_index: usize, current_x: f64) {
        let Some(switcher) = self.alt_tab_switcher.as_mut() else {
            return;
        };
        let count = switcher.entries.len();
        if count == 0 {
            return;
        }
        let index =
            crate::application::switcher::scrub_index(start_x, current_x, base_index, count);
        switcher.hovered = None;
        switcher.selected_index = index;
        switcher.scroll.snap_to(index as f64);
        switcher.highlight.snap_to(index as f64);
        switcher.stage = Tween::settled(1.0);
        switcher.presence = Tween::settled(1.0);
        switcher.closing = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dock_wraps_directly_to_the_first_or_last_icon() {
        assert_eq!(step_target(0.0, -1.0, 5), (4.0, true));
        assert_eq!(step_target(0.0, 1.0, 5), (1.0, false));
        assert_eq!(step_target(4.0, 1.0, 5), (0.0, true));
        assert_eq!(step_target(4.0, -1.0, 5), (3.0, false));
    }
}
