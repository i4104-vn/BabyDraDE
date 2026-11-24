//! Pure size and opacity math for Dynamic Island transitions.

/// Interpolates an integer size between two endpoints.
///
/// # Arguments
/// * `start` - Value at progress zero.
/// * `target` - Value at progress one.
/// * `eased` - Eased progress, which may overshoot or undershoot.
///
/// # Returns
/// The interpolated value, truncated toward zero like the GTK renderer.
pub(crate) fn interpolate(start: i32, target: i32, eased: f64) -> i32 {
    start + ((target - start) as f64 * eased) as i32
}

/// Child opacity while the capsule expands.
///
/// # Arguments
/// * `t` - Linear progress in `0.0..=1.0`.
///
/// # Returns
/// Opacity ramping up during the second half of the transition.
pub(crate) fn reveal_opacity(t: f64) -> f64 {
    ((t - 0.5) * 2.0).clamp(0.0, 1.0)
}

/// Capsule opacity while it collapses.
///
/// # Arguments
/// * `t` - Linear progress in `0.0..=1.0`.
///
/// # Returns
/// Opacity fading out during the second half of the transition.
pub(crate) fn collapse_opacity(t: f64) -> f64 {
    (1.0 - (t - 0.5) * 2.0).clamp(0.0, 1.0)
}

/// Child opacity while the capsule collapses.
///
/// # Arguments
/// * `t` - Linear progress in `0.0..=1.0`.
///
/// # Returns
/// Opacity fading out during the first half of the transition.
pub(crate) fn child_collapse_opacity(t: f64) -> f64 {
    (1.0 - t * 2.0).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolation_hits_both_endpoints() {
        assert_eq!(interpolate(22, 300, 0.0), 22);
        assert_eq!(interpolate(22, 300, 1.0), 300);
        assert_eq!(interpolate(300, 22, 0.5), 161);
    }

    #[test]
    fn reveal_and_collapse_opacities_are_complementary() {
        for step in 0..=10 {
            let t = step as f64 / 10.0;
            assert!((reveal_opacity(t) + collapse_opacity(t) - 1.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn child_collapse_finishes_at_half_progress() {
        assert_eq!(child_collapse_opacity(0.0), 1.0);
        assert_eq!(child_collapse_opacity(0.5), 0.0);
        assert_eq!(child_collapse_opacity(0.9), 0.0);
    }
}
