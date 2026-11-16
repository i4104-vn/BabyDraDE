/// Easing curves used by [`super::Tween`].
///
/// Curves take normalized linear progress (`0.0..=1.0`) and return normalized
/// eased progress. They are pure and allocation-free so they can be called
/// from render loops.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Easing {
    Linear,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    EaseInCubic,
    EaseOutCubic,
    /// Slow at both ends, fast in the middle. The default for UI transitions.
    #[default]
    EaseInOutCubic,
}

impl Easing {
    /// Apply the curve to linear progress.
    ///
    /// Values outside `0.0..=1.0` and `NaN` are safely clamped before computation.
    /// Implementations are algebraically simplified to avoid heavy `powi` or division
    /// operations in tight render loops.
    #[inline]
    pub fn apply(self, progress: f64) -> f64 {
        let t = if progress.is_nan() {
            0.0
        } else {
            progress.clamp(0.0, 1.0)
        };

        match self {
            Self::Linear => t,
            Self::EaseInQuad => t * t,
            Self::EaseOutQuad => {
                let p = 1.0 - t;
                1.0 - p * p
            }
            Self::EaseInOutQuad => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    let p = 1.0 - t;
                    1.0 - 2.0 * p * p
                }
            }
            Self::EaseInCubic => t * t * t,
            Self::EaseOutCubic => {
                let p = 1.0 - t;
                1.0 - p * p * p
            }
            Self::EaseInOutCubic => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    let p = 1.0 - t;
                    1.0 - 4.0 * p * p * p
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 1e-9,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn every_easing_has_stable_endpoints() {
        for easing in [
            Easing::Linear,
            Easing::EaseInQuad,
            Easing::EaseOutQuad,
            Easing::EaseInOutQuad,
            Easing::EaseInCubic,
            Easing::EaseOutCubic,
            Easing::EaseInOutCubic,
        ] {
            assert_close(easing.apply(0.0), 0.0);
            assert_close(easing.apply(1.0), 1.0);
        }
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
    fn ease_in_out_quad_is_smooth_and_symmetric() {
        let easing = Easing::EaseInOutQuad;
        assert_close(easing.apply(0.5), 0.5);
        assert!(easing.apply(0.25) < 0.25);
        assert!(easing.apply(0.75) > 0.75);
        assert_close(easing.apply(0.25) + easing.apply(0.75), 1.0);
    }

    #[test]
    fn ease_out_quad_matches_expected() {
        let easing = Easing::EaseOutQuad;
        assert_close(easing.apply(0.5), 0.75);
    }

    #[test]
    fn ease_out_cubic_matches_expected() {
        let easing = Easing::EaseOutCubic;
        assert_close(easing.apply(0.5), 0.875);
    }

    #[test]
    fn easing_clamps_out_of_range_and_nan_input() {
        assert_eq!(Easing::Linear.apply(-1.0), 0.0);
        assert_eq!(Easing::Linear.apply(2.0), 1.0);
        assert_eq!(Easing::Linear.apply(f64::NAN), 0.0);
    }
}
