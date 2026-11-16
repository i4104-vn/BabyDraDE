use std::time::{Duration, Instant};

use gtk4::gdk_pixbuf::Pixbuf;
use smithay::{
    desktop::Window,
    reexports::wayland_protocols::xdg::shell::server::xdg_toplevel,
    utils::{Logical, Point, Size},
    wayland::compositor::with_states,
};

use super::{window_ops::window_state, BabydraState};

/// Thời gian của một bước chuyển (mở switcher hoặc đổi app).
pub(crate) const ANIMATION_DURATION: Duration = Duration::from_millis(420);

/// Easing khởi đầu và kết thúc đều chậm, không giật.
pub(crate) fn ease_in_out_cubic(value: f64) -> f64 {
    if value < 0.5 {
        4.0 * value * value * value
    } else {
        1.0 - (-2.0 * value + 2.0).powi(3) / 2.0
    }
}

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
    pub(crate) transition_started: Instant,
    /// +1.0 khi chọn app kế tiếp, -1.0 khi Shift+Tab lùi lại.
    /// Dùng để animation stage trượt đúng hướng.
    pub(crate) scroll_direction: f64,
    /// Vị trí cuộn liên tục (đơn vị: slot) tại thời điểm bắt đầu animation.
    pub(crate) scroll_from: f64,
    /// Đích cuộn liên tục. Khác `selected_index` khi animation đang chạy.
    pub(crate) scroll_target: f64,
}

impl AltTabSwitcher {
    /// Vị trí chọn liên tục (có phần lẻ) dùng cho carousel bên trái.
    pub(crate) fn visual_selection(&self) -> f64 {
        let raw = (self.transition_started.elapsed().as_secs_f64()
            / ANIMATION_DURATION.as_secs_f64())
        .clamp(0.0, 1.0);
        self.scroll_from + (self.scroll_target - self.scroll_from) * ease_in_out_cubic(raw)
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
            // Lấy vị trí cuộn hiện tại trước khi thay đổi danh sách/mục chọn,
            // để nhiều lần bấm Tab liên tiếp vẫn nối tiếp mượt mà.
            let visual_before = switcher.visual_selection();
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
                switcher.transition_started = Instant::now();
                switcher.scroll_direction = direction;
                switcher.scroll_from = 0.0;
                switcher.scroll_target = 0.0;
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
                switcher.scroll_from = switcher.selected_index as f64;
                switcher.scroll_target = switcher.selected_index as f64;
            } else {
                // Cộng dồn vào đích thay vì vị trí hiện tại, nhờ vậy bấm Tab
                // liên tiếp không bị nhảy lùi về đầu bước trước.
                switcher.scroll_from = visual_before;
                switcher.scroll_target += direction;
            }
            switcher.transition_started = Instant::now();
            switcher.scroll_direction = direction;
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

                // Resolve the icon eagerly here (on the event-loop thread, before any
                // rendering happens) so the render path never has to call into GTK/GIO.
                // Wrapped in catch_unwind so a GTK/GIO panic inside icon resolution
                // cannot take down the whole compositor — the switcher simply falls
                // back to a coloured badge when the icon is unavailable.
                let icon_pixbuf =
                    std::panic::catch_unwind(|| ui_kit::icons::get_app_icon_pixbuf(&app_id, 32))
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

        let selected_index = if entries.len() == 1 {
            0
        } else if backwards {
            entries.len() - 1
        } else {
            1
        };
        let scroll = selected_index as f64;
        self.alt_tab_switcher = Some(AltTabSwitcher {
            entries,
            selected_index,
            transition_started: Instant::now(),
            scroll_direction: if backwards { -1.0 } else { 1.0 },
            scroll_from: scroll,
            scroll_target: scroll,
        });
    }
}
