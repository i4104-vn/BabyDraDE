use super::*;

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-7,
        "expected {expected}, got {actual}"
    );
}

const ALL_CURVES: [Easing; 31] = [
    Easing::Linear,
    Easing::EaseInQuad,
    Easing::EaseOutQuad,
    Easing::EaseInOutQuad,
    Easing::EaseInCubic,
    Easing::EaseOutCubic,
    Easing::EaseInOutCubic,
    Easing::EaseInSine,
    Easing::EaseOutSine,
    Easing::EaseInOutSine,
    Easing::EaseInQuart,
    Easing::EaseOutQuart,
    Easing::EaseInOutQuart,
    Easing::EaseInQuint,
    Easing::EaseOutQuint,
    Easing::EaseInOutQuint,
    Easing::EaseInExpo,
    Easing::EaseOutExpo,
    Easing::EaseInOutExpo,
    Easing::EaseInCirc,
    Easing::EaseOutCirc,
    Easing::EaseInOutCirc,
    Easing::EaseInBack,
    Easing::EaseOutBack,
    Easing::EaseOutBackSoft,
    Easing::EaseInOutBack,
    Easing::EaseInElastic,
    Easing::EaseOutElastic,
    Easing::EaseInOutElastic,
    Easing::EaseInBounce,
    Easing::EaseOutBounce,
];

#[test]
fn every_easing_has_stable_endpoints() {
    for easing in ALL_CURVES {
        assert_close(easing.apply(0.0), 0.0);
        assert_close(easing.apply(1.0), 1.0);
    }
    assert_close(Easing::EaseInOutBounce.apply(0.0), 0.0);
    assert_close(Easing::EaseInOutBounce.apply(1.0), 1.0);
    assert_close(Easing::Smoothstep.apply(0.0), 0.0);
    assert_close(Easing::Smoothstep.apply(1.0), 1.0);
    assert_close(Easing::Smootherstep.apply(0.0), 0.0);
    assert_close(Easing::Smootherstep.apply(1.0), 1.0);
}

#[test]
fn ease_in_out_cubic_is_smooth_and_symmetric() {
    let easing = Easing::EaseInOutCubic;
    assert_close(easing.apply(0.5), 0.5);
    assert!(easing.apply(0.25) < 0.25);
    assert!(easing.apply(0.75) > 0.75);
    assert_close(easing.apply(0.25) + easing.apply(0.75), 1.0);
}

#[test]
fn ease_in_out_sine_is_smooth_and_symmetric() {
    let easing = Easing::EaseInOutSine;
    assert_close(easing.apply(0.5), 0.5);
    assert_close(easing.apply(0.25) + easing.apply(0.75), 1.0);
}

#[test]
fn ease_out_back_overshoots() {
    let easing = Easing::EaseOutBack;
    let mid_val = easing.apply(0.7);
    assert!(
        mid_val > 1.0,
        "EaseOutBack should overshoot target, got {mid_val}"
    );
    assert_close(easing.apply(1.0), 1.0);
}

#[test]
fn soft_back_overshoots_less_than_standard_back() {
    let t = 0.7;
    assert!(Easing::EaseOutBackSoft.apply(t) > 1.0);
    assert!(Easing::EaseOutBackSoft.apply(t) < Easing::EaseOutBack.apply(t));
    assert_close(Easing::EaseOutBackSoft.apply(1.0), 1.0);
}

#[test]
fn smoothstep_and_smootherstep_are_symmetric() {
    assert_close(Easing::Smoothstep.apply(0.5), 0.5);
    assert_close(Easing::Smootherstep.apply(0.5), 0.5);
    assert_close(
        Easing::Smoothstep.apply(0.25) + Easing::Smoothstep.apply(0.75),
        1.0,
    );
    assert_close(
        Easing::Smootherstep.apply(0.25) + Easing::Smootherstep.apply(0.75),
        1.0,
    );
}

#[test]
fn easing_clamps_out_of_range_and_nan_input() {
    assert_eq!(Easing::Linear.apply(-1.0), 0.0);
    assert_eq!(Easing::Linear.apply(2.0), 1.0);
    assert_eq!(Easing::Linear.apply(f64::NAN), 0.0);
    assert_eq!(Easing::EaseOutBounce.apply(-0.5), 0.0);
    assert_eq!(Easing::EaseOutBounce.apply(1.5), 1.0);
}
