//! Stagger state and per-element progress math.

use std::time::Duration;

use crate::{Easing, Tween};

/// A staggered sequence of elements sharing one clock.
#[derive(Clone, Copy, Debug)]
pub struct Stagger {
    pub(super) clock: Tween,
    lead_span: f64,
    start: f64,
    span: f64,
    step: f64,
}

impl Stagger {
    /// Creates a resting sequence. All fractions are relative to the timeline.
    ///
    /// # Arguments
    /// * `lead_span` - Fraction of the timeline reserved for the leading element.
    /// * `start` - Fraction at which the first sequenced element starts.
    /// * `span` - Fraction each sequenced element takes to complete.
    /// * `step` - Fraction added to the start of each subsequent element.
    ///
    /// # Returns
    /// A sequence that rests at zero progress.
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

    /// Raw, linear local progress `0..=1` of one element.
    ///
    /// Index `0` is the leading element; index `n` (`n >= 1`) is the `n`-th
    /// sequenced element, delayed by `start + (n - 1) * step`.
    ///
    /// # Arguments
    /// * `index` - Zero-based element index.
    ///
    /// # Returns
    /// Clamped local progress for that element.
    #[inline]
    pub fn progress(&self, index: usize) -> f64 {
        let t = self.clock.value().clamp(0.0, 1.0);
        let (delay, span) = if index == 0 {
            (0.0, self.lead_span)
        } else {
            (self.start + (index - 1) as f64 * self.step, self.span)
        };
        ((t - delay) / span).clamp(0.0, 1.0)
    }
}
