//! Compositor-side window animation lifecycle.

use super::*;

impl BabydraState {
    /// Starts the closing animation for a window, immediately delegating focus to the next window.
    pub(crate) fn close_window(&mut self, window: &Window) {
        {
            let mut state_guard = lock_window_state(window);
            if let Some(WindowAnimation::Closing(_)) = state_guard.animation {
                return;
            }
            state_guard.animation = Some(WindowAnimation::Closing(Tween::new(
                0.0,
                1.0,
                WINDOW_CLOSE_DURATION,
                Easing::EaseInOutCubic,
            )));
        }

        // Immediately update active window and transfer keyboard focus to next window
        let next_top = self
            .windows.space
            .elements()
            .rev()
            .find(|w| *w != window && !is_window_exiting(w))
            .cloned();
        self.update_active_window(next_top.as_ref());
        if let Some(ref next_win) = next_top {
            if let Some(keyboard) = self.wayland.seat.get_keyboard() {
                if let Some(toplevel) = next_win.toplevel() {
                    keyboard.set_focus(
                        self,
                        Some(toplevel.wl_surface().clone()),
                        SERIAL_COUNTER.next_serial(),
                    );
                }
            }
        }
    }

    /// Progresses active window animations and completes transitions.
    pub(crate) fn update_window_animations(&mut self) {
        // A closing switcher stays alive until its fade/slide finishes.
        let switcher_done = self
            .overlays.alt_tab_switcher
            .as_ref()
            .is_some_and(|switcher| switcher.closing && !switcher.presence.is_animating());
        if switcher_done {
            self.overlays.alt_tab_switcher = None;
        }
        if let Some(switcher) = self.overlays.alt_tab_switcher.as_mut() {
            switcher.tick_key_repeat();
        }

        let mut to_close = Vec::new();
        let mut finished_opening = Vec::new();
        let mut finished_minimizing = Vec::new();
        let mut finished = Vec::new();

        for window in self.windows.space.elements() {
            let state_guard = lock_window_state(window);
            if let Some(ref anim) = state_guard.animation {
                match anim {
                    WindowAnimation::Opening(tween) => {
                        if !tween.is_animating() {
                            finished_opening.push(window.clone());
                        }
                    }
                    WindowAnimation::Closing(tween) => {
                        if !tween.is_animating() {
                            to_close.push(window.clone());
                        }
                    }
                    WindowAnimation::Minimizing(tween) => {
                        if !tween.is_animating() {
                            finished_minimizing.push(window.clone());
                        }
                    }
                    WindowAnimation::Restoring(tween) => {
                        if !tween.is_animating() {
                            finished.push(window.clone());
                        }
                    }
                    WindowAnimation::Transition { tween, .. } => {
                        if !tween.is_animating() {
                            finished.push(window.clone());
                        }
                    }
                }
            }
        }

        for window in finished_opening {
            let mut state_guard = lock_window_state(&window);
            state_guard.animation = None;
        }

        for window in finished {
            let mut state_guard = lock_window_state(&window);
            state_guard.animation = None;
        }

        for window in finished_minimizing {
            {
                let mut state_guard = lock_window_state(&window);
                state_guard.animation = None;
            }
            self.windows.space.unmap_elem(&window);
            self.windows.space.refresh();
        }

        for window in to_close {
            {
                let mut state_guard = lock_window_state(&window);
                state_guard.animation = None;
            }
            if let Some(toplevel) = window.toplevel() {
                toplevel.send_close();
            }
            self.windows.space.unmap_elem(&window);
        }

        // Always clean up, including when the last window closed, so the
        // active-window state file is cleared for the topbar app label.
        self.cleanup_windows();
    }
}
