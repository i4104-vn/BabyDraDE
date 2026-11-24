//! Exposes stagger progress to renderers.

use super::handler::Stagger;
use crate::Tween;

impl Stagger {
    /// Current master clock value.
    #[inline]
    pub fn value(&self) -> f64 {
        self.clock.value()
    }

    /// Target of the master clock.
    #[inline]
    pub fn target(&self) -> f64 {
        self.clock.target()
    }

    /// Whether the master clock is still animating.
    #[inline]
    pub fn is_animating(&self) -> bool {
        self.clock.is_animating()
    }

    /// Access to the underlying clock, e.g. for cache-key aggregation.
    #[inline]
    pub fn tween(&self) -> &Tween {
        &self.clock
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn elements_reveal_in_order() {
        let mut sequence = Stagger::new(0.2, 0.2, 0.2, 0.1);
        sequence.reveal(std::time::Duration::from_millis(20));
        sleep(std::time::Duration::from_millis(10));
        let lead = sequence.progress(0);
        let first = sequence.progress(1);
        let second = sequence.progress(2);
        assert!(lead >= first);
        assert!(first >= second);
        assert_eq!(lead, 1.0);
    }

    #[test]
    fn resting_sequence_reports_zero_progress() {
        let sequence = Stagger::new(0.2, 0.2, 0.2, 0.1);
        assert_eq!(sequence.progress(0), 0.0);
        assert_eq!(sequence.progress(3), 0.0);
    }

    #[test]
    fn snap_to_settles_without_animating() {
        let mut sequence = Stagger::new(0.2, 0.2, 0.2, 0.1);
        sequence.reveal(std::time::Duration::from_secs(1));
        sequence.snap_to(1.0);
        assert!(!sequence.is_animating());
        assert_eq!(sequence.progress(0), 1.0);
    }
}
