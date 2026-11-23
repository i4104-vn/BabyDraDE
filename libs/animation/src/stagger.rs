//! Sequential reveal driven by a single master clock.
//!
//! Maps one linear `0..=1` timeline onto independent, delayed progress values
//! so a leading element (e.g. an input row) settles first and every following
//! element arrives one after another. Callers decide what to do with the raw
//! progress — fade it through [`Easing::EaseOutCubic`], offset it through
//! [`Easing::EaseOutBack`], and so on.

use std::time::Duration;

use crate::{Easing, Tween};

/// A staggered sequence of elements sharing one clock.
#[derive(Clone, Copy, Debug)]
pub struct Stagger {
    clock: Tween,
    /// Fraction of the timeline reserved for element 0.
    lead_span: f64,
    /// Fraction at which element 1 starts.
    start: f64,
    /// Fraction each sequence element takes to complete.
    span: f64,
    /// Fraction added to the start of each subsequent element.
    step: f64,
}

impl Stagger {
    /// Creates a resting sequence. All fractions are relative to the timeline.
    #[inline]
    pub fn new(lead_span: f64, start: f64, span: f64, step: f64) -> Self {
        Self {
            clock: Tween::settled(0.0),
            lead_span: lead_span.max(f64::EPSILON),
            start,
            span: span.max(f64::EPSILON),
            step,
        }
    }

    /// Runs the timeline forward so elements reveal in order.
    #[inline]
    pub fn reveal(&mut self, duration: Duration) {
        if self.clock.target() != 1.0 {
            self.clock.retarget(1.0, duration, Easing::Linear);
        }
    }

    /// Runs the timeline backward so elements leave in reverse order.
    #[inline]
    pub fn dismiss(&mut self, duration: Duration) {
        if self.clock.target() != 0.0 {
            self.clock.retarget(0.0, duration, Easing::Linear);
        }
    }

    /// Jumps to a resting endpoint without animating.
    #[inline]
    pub fn snap_to(&mut self, value: f64) {
        self.clock.snap_to(value);
    }

    #[inline]
    pub fn value(&self) -> f64 {
        self.clock.value()
    }

    #[inline]
    pub fn target(&self) -> f64 {
        self.clock.target()
    }

    #[inline]
    pub fn is_animating(&self) -> bool {
        self.clock.is_animating()
    }

    /// Access to the underlying clock, e.g. for cache-key aggregation.
    #[inline]
    pub fn tween(&self) -> &Tween {
        &self.clock
    }

    /// Raw, linear local progress `0..=1` of one element.
    ///
    /// Index `0` is the leading element; index `n` (`n >= 1`) is the `n`-th
    /// sequenced element, delayed by `start + (n - 1) * step`.
    #[inline]
    pub fn progress(&self, index: usize) -> f64 {
        let t = self.value().clamp(0.0, 1.0);
        let (delay, span) = if index == 0 {
            (0.0, self.lead_span)
        } else {
            (self.start + (index - 1) as f64 * self.step, self.span)
        };
        ((t - delay) / span).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn elements_reveal_in_order() {
        let mut sequence = Stagger::new(0.2, 0.2, 0.2, 0.1);
        sequence.reveal(Duration::from_millis(20));
        sleep(Duration::from_millis(10));
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
        sequence.reveal(Duration::from_secs(1));
        sequence.snap_to(1.0);
        assert!(!sequence.is_animating());
        assert_eq!(sequence.progress(0), 1.0);
    }
}
