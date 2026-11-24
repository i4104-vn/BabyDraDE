use animation::Tween;
use smithay::utils::SERIAL_COUNTER;

use crate::adapters::compositor::state::BabydraState;

impl BabydraState {
    /// Opens the switcher for a finger drag and returns the index to scrub from.
    pub(crate) fn begin_switcher_scrub(&mut self) -> Option<usize> {
        if self.overlays.alt_tab_switcher.is_none() {
            self.cycle_alt_tab(false);
        }
        self.overlays
            .alt_tab_switcher
            .as_ref()
            .map(|switcher| switcher.selected_index)
    }

    /// Focuses the current switcher entry and starts the close animation.
    pub(crate) fn commit_switcher_selection(&mut self) {
        let window = self
            .overlays
            .alt_tab_switcher
            .as_ref()
            .and_then(|switcher| {
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
        if let Some(switcher) = self.overlays.alt_tab_switcher.as_mut() {
            switcher.begin_close();
        }
    }

    /// Moves the active switcher entry with the finger. `steps` is the number
    /// of icons travelled since the drag started (may be negative).
    pub(crate) fn scrub_switcher(&mut self, start_x: f64, base_index: usize, current_x: f64) {
        let Some(switcher) = self.overlays.alt_tab_switcher.as_mut() else {
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
