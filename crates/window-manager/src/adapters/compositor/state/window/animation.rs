use animation::{
    Easing, Tween, WINDOW_CLOSE_DURATION, WINDOW_GEOMETRY_DURATION, WINDOW_MINIMIZE_DURATION,
    WINDOW_OPEN_DURATION, WINDOW_RESTORE_DURATION,
};
use smithay::{
    desktop::Window,
    utils::{Logical, Point, Rectangle, Size, SERIAL_COUNTER},
};

use super::state_data::lock_window_state;
use crate::adapters::compositor::state::BabydraState;

/// Start and end rectangles (in space/logical coordinates) of a geometry
/// transition such as fullscreen or maximize enter/exit.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GeometryTransition {
    pub(crate) from: Rectangle<f64, Logical>,
    pub(crate) to: Rectangle<f64, Logical>,
}

/// Tracks in-progress window transitions.
///
/// All of them are render-only: the real window geometry is changed right away
/// and the renderer draws a temporary, interpolated element until the tween
/// finishes. `Minimizing` is the exception in that it keeps the window mapped
/// until the shrink completes so it can actually be seen collapsing.
#[derive(Debug, Clone)]
pub(crate) enum WindowAnimation {
    Opening(Tween),
    Closing(Tween),
    Minimizing(Tween),
    Restoring(Tween),
    Transition {
        tween: Tween,
        geometry: GeometryTransition,
    },
}

use crate::domain::geometry::{Point as PurePoint, Rectangle as PureRectangle, Size as PureSize};
use crate::domain::WindowVisualState;

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

fn lerp_rect(
    from: Rectangle<f64, Logical>,
    to: Rectangle<f64, Logical>,
    progress: f64,
) -> Rectangle<f64, Logical> {
    Rectangle::new(
        Point::from((
            from.loc.x + (to.loc.x - from.loc.x) * progress,
            from.loc.y + (to.loc.y - from.loc.y) * progress,
        )),
        Size::from((
            (from.size.w + (to.size.w - from.size.w) * progress).max(1.0),
            (from.size.h + (to.size.h - from.size.h) * progress).max(1.0),
        )),
    )
}

/// Converts a Smithay render rectangle to the pure domain representation at
/// the adapter boundary.
fn to_pure_rect(rect: Rectangle<f64, Logical>) -> PureRectangle<f64> {
    PureRectangle::new(
        PurePoint::from((rect.loc.x, rect.loc.y)),
        PureSize::from((rect.size.w, rect.size.h)),
    )
}

/// Computes the current render parameters for a window transition.
pub(crate) fn window_animation_frame(window: &Window) -> WindowVisualState {
    let guard = lock_window_state(window);
    match &guard.animation {
        Some(WindowAnimation::Opening(tween)) => {
            let p = tween.value().clamp(0.0, 1.0);
            // Expands smoothly from 85% to 100% size while fading in from 0% to 100%
            WindowVisualState {
                active: true,
                scale_factor: 0.85 + 0.15 * p,
                alpha_factor: p as f32,
                target_rect: None,
            }
        }
        Some(WindowAnimation::Closing(tween)) => {
            let p = tween.value().clamp(0.0, 1.0);
            // Shrinks towards center (100% down to 72%) and dims gently before fading out completely
            WindowVisualState {
                active: true,
                scale_factor: 1.0 - 0.28 * p,
                alpha_factor: ((1.0 - p * p) as f32).max(0.0),
                target_rect: None,
            }
        }
        Some(WindowAnimation::Minimizing(tween)) => {
            let p = tween.value().clamp(0.0, 1.0);
            // Collapses into a point at the window center while fading out.
            WindowVisualState {
                active: true,
                scale_factor: (1.0 - 0.94 * p).max(0.02),
                alpha_factor: ((1.0 - p * p) as f32).max(0.0),
                target_rect: None,
            }
        }
        Some(WindowAnimation::Restoring(tween)) => {
            let p = tween.value().clamp(0.0, 1.0);
            // Grows out of the center while fading in.
            WindowVisualState {
                active: true,
                scale_factor: 0.25 + 0.75 * p,
                alpha_factor: p as f32,
                target_rect: None,
            }
        }
        Some(WindowAnimation::Transition { tween, geometry }) => {
            let p = tween.value().clamp(0.0, 1.0);
            WindowVisualState {
                active: true,
                scale_factor: 1.0,
                alpha_factor: 1.0,
                target_rect: Some(to_pure_rect(lerp_rect(geometry.from, geometry.to, p))),
            }
        }
        None => WindowVisualState::idle(),
    }
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
            .space
            .elements()
            .rev()
            .find(|w| *w != window && !is_window_exiting(w))
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

    /// Progresses active window animations and completes transitions.
    pub(crate) fn update_window_animations(&mut self) {
        // A closing switcher stays alive until its fade/slide finishes.
        let switcher_done = self
            .alt_tab_switcher
            .as_ref()
            .is_some_and(|switcher| switcher.closing && !switcher.presence.is_animating());
        if switcher_done {
            self.alt_tab_switcher = None;
        }
        if let Some(switcher) = self.alt_tab_switcher.as_mut() {
            switcher.tick_key_repeat();
        }

        let mut to_close = Vec::new();
        let mut finished_opening = Vec::new();
        let mut finished_minimizing = Vec::new();
        let mut finished = Vec::new();

        for window in self.space.elements() {
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
            self.space.unmap_elem(&window);
            self.space.refresh();
        }

        for window in to_close {
            {
                let mut state_guard = lock_window_state(&window);
                state_guard.animation = None;
            }
            if let Some(toplevel) = window.toplevel() {
                toplevel.send_close();
            }
            self.space.unmap_elem(&window);
        }

        // Always clean up, including when the last window closed, so the
        // active-window state file is cleared for the topbar app label.
        self.cleanup_windows();
    }
}

#[cfg(test)]
mod tests {
    use super::super::state_data::WindowStateData;
    use super::*;
    use std::time::Duration;

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

        let minimize_tween = Tween::new(0.0, 1.0, Duration::ZERO, Easing::Linear);
        state.animation = Some(WindowAnimation::Minimizing(minimize_tween));
        assert!(matches!(
            state.animation,
            Some(WindowAnimation::Minimizing(_))
        ));

        let restore_tween = Tween::new(0.0, 1.0, Duration::ZERO, Easing::Linear);
        state.animation = Some(WindowAnimation::Restoring(restore_tween));
        assert!(matches!(
            state.animation,
            Some(WindowAnimation::Restoring(_))
        ));

        let geometry = GeometryTransition {
            from: Rectangle::new(Point::from((0.0, 0.0)), Size::from((1920.0, 1080.0))),
            to: Rectangle::new(Point::from((100.0, 100.0)), Size::from((800.0, 600.0))),
        };
        state.animation = Some(WindowAnimation::Transition {
            tween: Tween::new(0.0, 1.0, Duration::ZERO, Easing::Linear),
            geometry,
        });
        assert!(matches!(
            state.animation,
            Some(WindowAnimation::Transition { .. })
        ));

        let mid = lerp_rect(geometry.from, geometry.to, 0.5);
        assert_eq!(mid.loc, Point::from((50.0, 50.0)));
        assert_eq!(mid.size, Size::from((1360.0, 840.0)));
    }
}
