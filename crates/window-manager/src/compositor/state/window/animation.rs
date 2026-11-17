use std::time::Duration;

use animation::{Easing, Tween};
use smithay::{desktop::Window, utils::SERIAL_COUNTER};

use super::state_data::window_state;
use crate::compositor::state::BabydraState;

/// Tracks in-progress window transitions (open zoom-in, close zoom-out + fade).
#[derive(Debug, Clone)]
pub(crate) enum WindowAnimation {
    Opening(Tween),
    Closing(Tween),
}

/// Checks if a window is currently running its closing shrink & fade animation.
pub(crate) fn is_window_closing(window: &Window) -> bool {
    let guard = window_state(window).lock().unwrap();
    matches!(guard.animation, Some(WindowAnimation::Closing(_)))
}

/// Initiates an opening transition when the window's first buffer is committed.
pub(crate) fn start_window_opening(window: &Window) {
    let mut state_guard = window_state(window).lock().unwrap();
    if !state_guard.has_opened && window.bbox().size.w > 0 && window.bbox().size.h > 0 {
        state_guard.has_opened = true;
        state_guard.animation = Some(WindowAnimation::Opening(Tween::new(
            0.0,
            1.0,
            Duration::from_millis(crate::enums::constants::WINDOW_OPEN_ANIM_MS),
            Easing::EaseOutCubic,
        )));
    }
}

/// Computes the current (scale_factor, alpha_factor) for rendering window transitions.
pub(crate) fn window_animation_factors(window: &Window) -> (f64, f32) {
    let guard = window_state(window).lock().unwrap();
    match &guard.animation {
        Some(WindowAnimation::Opening(tween)) => {
            let p = tween.value().clamp(0.0, 1.0);
            // Expands smoothly from 85% to 100% size while fading in from 0% to 100%
            let scale = 0.85 + 0.15 * p;
            let alpha = p as f32;
            (scale, alpha)
        }
        Some(WindowAnimation::Closing(tween)) => {
            let p = tween.value().clamp(0.0, 1.0);
            // Shrinks towards center (100% down to 72%) and dims gently before fading out completely
            let scale = 1.0 - 0.28 * p;
            let alpha = ((1.0 - p * p) as f32).max(0.0);
            (scale, alpha)
        }
        None => (1.0, 1.0),
    }
}

impl BabydraState {
    /// Starts the closing animation for a window, immediately delegating focus to the next window.
    pub(crate) fn close_window(&mut self, window: &Window) {
        {
            let mut state_guard = window_state(window).lock().unwrap();
            if let Some(WindowAnimation::Closing(_)) = state_guard.animation {
                return;
            }
            state_guard.animation = Some(WindowAnimation::Closing(Tween::new(
                0.0,
                1.0,
                Duration::from_millis(crate::enums::constants::WINDOW_CLOSE_ANIM_MS),
                Easing::EaseOutCubic,
            )));
        }

        // Immediately update active window and transfer keyboard focus to next window
        let next_top = self
            .space
            .elements()
            .rev()
            .find(|w| *w != window && !is_window_closing(w))
            .cloned();
        self.update_active_window(next_top.as_ref());
        if let Some(ref next_win) = next_top {
            if let Some(keyboard) = self.seat.get_keyboard() {
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

    /// Progresses active window animations and completes closing transitions.
    pub(crate) fn update_window_animations(&mut self) {
        let mut to_close = Vec::new();
        let mut finished_opening = Vec::new();

        for window in self.space.elements() {
            let state_guard = window_state(window).lock().unwrap();
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
                }
            }
        }

        for window in finished_opening {
            let mut state_guard = window_state(&window).lock().unwrap();
            state_guard.animation = None;
        }

        for window in to_close {
            {
                let mut state_guard = window_state(&window).lock().unwrap();
                state_guard.animation = None;
            }
            if let Some(toplevel) = window.toplevel() {
                toplevel.send_close();
            }
            self.space.unmap_elem(&window);
        }

        if !self.windows.is_empty() {
            self.cleanup_windows();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::state_data::WindowStateData;
    use super::*;

    #[test]
    fn test_window_animation_factors_and_states() {
        let mut state = WindowStateData::default();
        assert!(state.animation.is_none());
        assert!(!state.has_opened);

        let open_tween = Tween::new(0.0, 1.0, Duration::ZERO, Easing::Linear);
        state.animation = Some(WindowAnimation::Opening(open_tween));
        assert!(matches!(state.animation, Some(WindowAnimation::Opening(_))));

        let close_tween = Tween::new(0.0, 1.0, Duration::ZERO, Easing::Linear);
        state.animation = Some(WindowAnimation::Closing(close_tween));
        assert!(matches!(state.animation, Some(WindowAnimation::Closing(_))));
    }
}
