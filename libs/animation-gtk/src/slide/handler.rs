//! Pure margin math for frame-synced slide transitions.

/// Direction axis for a slide transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlideDirection {
    Down,
    Up,
    Left,
    Right,
}

/// Initial margin of a widget entering from a given direction.
///
/// # Arguments
/// * `original` - Margin the widget rests at.
/// * `distance` - Travel distance in pixels.
///
/// # Returns
/// The clamped margin at the start of the transition.
pub(crate) fn start_margin(original: i32, distance: i32) -> i32 {
    (original - distance).max(0)
}

/// Margin of an entering widget at a given eased progress.
///
/// # Arguments
/// * `original` - Margin the widget rests at.
/// * `distance` - Travel distance in pixels.
/// * `eased` - Eased progress in `0.0..=1.0`.
///
/// # Returns
/// The interpolated, non-negative margin.
pub(crate) fn entering_margin(original: i32, distance: i32, eased: f64) -> i32 {
    (original - distance + (distance as f64 * eased) as i32).max(0)
}

/// Margin of an exiting widget at a given eased progress.
///
/// # Arguments
/// * `original` - Margin the widget rests at.
/// * `distance` - Travel distance in pixels.
/// * `eased` - Eased progress in `0.0..=1.0`.
///
/// # Returns
/// The interpolated, non-negative margin.
pub(crate) fn exiting_margin(original: i32, distance: i32, eased: f64) -> i32 {
    (original - (distance as f64 * eased) as i32).max(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entering_margin_reaches_rest_at_full_progress() {
        assert_eq!(entering_margin(40, 30, 1.0), 40);
        assert_eq!(entering_margin(40, 30, 0.0), start_margin(40, 30));
    }

    #[test]
    fn margins_never_become_negative_on_large_distances() {
        assert_eq!(start_margin(10, 50), 0);
        assert_eq!(entering_margin(10, 50, 0.5), 0);
        assert_eq!(entering_margin(10, 50, 1.0), 10);
        assert_eq!(exiting_margin(10, 50, 0.5), 0);
    }

    #[test]
    fn exiting_margin_decreases_monotonically() {
        let mut previous = i32::MAX;
        for step in 0..=10 {
            let margin = exiting_margin(100, 40, step as f64 / 10.0);
            assert!(margin <= previous);
            previous = margin;
        }
        assert_eq!(previous, 60);
    }
}
