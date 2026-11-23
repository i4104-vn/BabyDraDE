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
