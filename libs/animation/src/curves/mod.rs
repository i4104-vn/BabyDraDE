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
    /// Back curve with a gentler overshoot, used by capsule transitions.
    EaseOutBackSoft,
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
            Self::EaseOutBackSoft => overshoot::ease_out_back_soft(t),
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
mod tests;
