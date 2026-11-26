//! Compositor-side window animation lifecycle.

use animation::{
    Easing, Tween, WINDOW_CLOSE_DURATION, WINDOW_GEOMETRY_DURATION, WINDOW_MINIMIZE_DURATION,
    WINDOW_OPEN_DURATION, WINDOW_RESTORE_DURATION,
};
use smithay::{
    desktop::Window,
    utils::{Logical, Rectangle, SERIAL_COUNTER},
};

use crate::app::state::BabydraState;
use crate::modules::window::state_data::lock_window_state;

pub(crate) use super::{GeometryTransition, WindowAnimation};

/// Checks if a window is leaving the screen (closing or minimizing) and should
/// therefore be excluded from hit-testing, focus and window switching.
pub(crate) fn is_window_exiting(window: &Window) -> bool {
    let guard = lock_window_state(window);
    matches!(
        guard.animation,
        Some(WindowAnimation::Closing(_)) | Some(WindowAnimation::Minimizing(_))
    )
}

/// Initiates an opening transition when the window's first buffer is committed.
pub(crate) fn start_window_opening(window: &Window) {
    let mut state_guard = lock_window_state(window);
    if !state_guard.has_opened && window.bbox().size.w > 0 && window.bbox().size.h > 0 {
        state_guard.has_opened = true;
        state_guard.animation = Some(WindowAnimation::Opening(Tween::new(
            0.0,
            1.0,
            WINDOW_OPEN_DURATION,
            Easing::EaseInOutCubic,
        )));
    }
}

/// Starts the shrink animation played before a minimized window is unmapped.
pub(crate) fn start_window_minimizing(window: &Window) {
    let mut state_guard = lock_window_state(window);
    state_guard.animation = Some(WindowAnimation::Minimizing(Tween::new(
        0.0,
        1.0,
        WINDOW_MINIMIZE_DURATION,
        Easing::EaseInOutCubic,
    )));
}

/// Cancels a pending minimize animation when the window is restored quickly.
pub(crate) fn cancel_minimizing_animation(window: &Window) {
    let mut state_guard = lock_window_state(window);
    if matches!(state_guard.animation, Some(WindowAnimation::Minimizing(_))) {
        state_guard.animation = None;
    }
}

/// Starts the grow animation used when a minimized window is restored.
pub(crate) fn start_window_restoring(window: &Window) {
    let mut state_guard = lock_window_state(window);
    state_guard.animation = Some(WindowAnimation::Restoring(Tween::new(
        0.0,
        1.0,
        WINDOW_RESTORE_DURATION,
        Easing::EaseInOutCubic,
    )));
}

/// Starts a smooth geometry transition between two rectangles (e.g. entering
/// or leaving fullscreen/maximized state).
pub(crate) fn start_window_transition(
    window: &Window,
    from: Rectangle<i32, Logical>,
    to: Rectangle<i32, Logical>,
) {
    let mut state_guard = lock_window_state(window);
    // Never interrupt an animation that is taking the window off screen.
    if matches!(
        state_guard.animation,
        Some(WindowAnimation::Closing(_)) | Some(WindowAnimation::Minimizing(_))
    ) {
        return;
    }
    state_guard.animation = Some(WindowAnimation::Transition {
        tween: Tween::new(0.0, 1.0, WINDOW_GEOMETRY_DURATION, Easing::EaseInOutCubic),
        geometry: GeometryTransition {
            from: from.to_f64(),
            to: to.to_f64(),
        },
    });
}

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
            .windows
            .space
            .elements()
            .rev()
            .find(|w| *w != window && !is_window_exiting(w))
            .cloned();
        self.update_active_window(next_top.as_ref());
        if let Some(ref next_win) = next_top {
            if let Some(keyboard) = self.wayland.seat().get_keyboard() {
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
            .overlays
            .alt_tab_switcher
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
