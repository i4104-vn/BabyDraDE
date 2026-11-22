pub mod hermite;
pub mod overshoot;
pub mod standard;

/// Easing curves used by [`crate::Tween`].
///
/// Curves take normalized linear progress (`0.0..=1.0`) and return normalized
/// eased progress. They are pure, allocation-free, and numerically stable so
/// they can be called every frame inside high-refresh-rate render loops.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Easing {
    /// Constant velocity.
    Linear,

    // Quadratic curves
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,

    // Cubic curves
    EaseInCubic,
    EaseOutCubic,
    /// Slow at both ends, fast in the middle. The default for UI transitions.
    #[default]
    EaseInOutCubic,

    // Sine curves
    EaseInSine,
    EaseOutSine,
    EaseInOutSine,

    // Quartic curves
    EaseInQuart,
    EaseOutQuart,
    EaseInOutQuart,

    // Quintic curves
    EaseInQuint,
    EaseOutQuint,
    EaseInOutQuint,

    // Exponential curves
    EaseInExpo,
    EaseOutExpo,
    EaseInOutExpo,

    // Circular curves
    EaseInCirc,
    EaseOutCirc,
    EaseInOutCirc,

    // Back curves
    EaseInBack,
    EaseOutBack,
    EaseInOutBack,

    // Elastic curves
    EaseInElastic,
    EaseOutElastic,
    EaseInOutElastic,

    // Bounce curves
    EaseInBounce,
    EaseOutBounce,
    EaseInOutBounce,

    // Hermite polynomials
    Smoothstep,
    Smootherstep,
}

impl Easing {
    /// Apply the curve to linear progress.
    #[inline]
    pub fn apply(self, progress: f64) -> f64 {
        let t = if progress.is_nan() {
            0.0
        } else {
            progress.clamp(0.0, 1.0)
        };

        match self {
            Self::Linear => standard::linear(t),

            Self::EaseInQuad => standard::ease_in_quad(t),
            Self::EaseOutQuad => standard::ease_out_quad(t),
            Self::EaseInOutQuad => standard::ease_in_out_quad(t),

            Self::EaseInCubic => standard::ease_in_cubic(t),
            Self::EaseOutCubic => standard::ease_out_cubic(t),
            Self::EaseInOutCubic => standard::ease_in_out_cubic(t),

            Self::EaseInSine => standard::ease_in_sine(t),
            Self::EaseOutSine => standard::ease_out_sine(t),
            Self::EaseInOutSine => standard::ease_in_out_sine(t),

            Self::EaseInQuart => standard::ease_in_quart(t),
            Self::EaseOutQuart => standard::ease_out_quart(t),
            Self::EaseInOutQuart => standard::ease_in_out_quart(t),

            Self::EaseInQuint => standard::ease_in_quint(t),
            Self::EaseOutQuint => standard::ease_out_quint(t),
            Self::EaseInOutQuint => standard::ease_in_out_quint(t),

            Self::EaseInExpo => standard::ease_in_expo(t),
            Self::EaseOutExpo => standard::ease_out_expo(t),
            Self::EaseInOutExpo => standard::ease_in_out_expo(t),

            Self::EaseInCirc => standard::ease_in_circ(t),
            Self::EaseOutCirc => standard::ease_out_circ(t),
            Self::EaseInOutCirc => standard::ease_in_out_circ(t),

            Self::EaseInBack => overshoot::ease_in_back(t),
            Self::EaseOutBack => overshoot::ease_out_back(t),
            Self::EaseInOutBack => overshoot::ease_in_out_back(t),

            Self::EaseInElastic => overshoot::ease_in_elastic(t),
            Self::EaseOutElastic => overshoot::ease_out_elastic(t),
            Self::EaseInOutElastic => overshoot::ease_in_out_elastic(t),

            Self::EaseInBounce => overshoot::ease_in_bounce(t),
            Self::EaseOutBounce => overshoot::ease_out_bounce(t),
            Self::EaseInOutBounce => overshoot::ease_in_out_bounce(t),

            Self::Smoothstep => hermite::smoothstep(t),
            Self::Smootherstep => hermite::smootherstep(t),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 1e-7,
            "expected {expected}, got {actual}"
        );
    }

    const ALL_CURVES: [Easing; 30] = [
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
}
