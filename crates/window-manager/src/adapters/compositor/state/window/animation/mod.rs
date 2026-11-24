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


mod frame;
pub(crate) use frame::window_animation_frame;
mod lifecycle;
