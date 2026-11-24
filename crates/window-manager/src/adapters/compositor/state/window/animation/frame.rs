//! Animation frame evaluation and interpolation helpers.

use super::*;

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
