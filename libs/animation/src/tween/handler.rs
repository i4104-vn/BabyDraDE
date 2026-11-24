//! Tween state and pure interpolation math.

use std::time::{Duration, Instant};

use crate::curves::Easing;
use crate::time::clock::tick_between;
use crate::time::constants::DEFAULT_DURATION;
use crate::time::frame::frame_index_since;

/// A clock-driven interpolation between two `f64` values.
#[derive(Clone, Copy, Debug)]
pub struct Tween {
    pub(super) started_at: Instant,
    pub(super) duration: Duration,
    pub(super) from: f64,
    pub(super) to: f64,
    pub(super) easing: Easing,
}

impl Default for Tween {
    #[inline]
    fn default() -> Self {
        Self::settled(0.0)
    }
}

impl Tween {
    /// Create a tween that starts immediately and animates `from -> to`.
    #[inline]
    pub fn new(from: f64, to: f64, duration: Duration, easing: Easing) -> Self {
        Self::new_at(from, to, duration, easing, Instant::now())
    }

    /// Create a tween with an explicit start time.
    ///
    /// # Arguments
    /// * `from` - Starting value.
    /// * `to` - Target value.
    /// * `duration` - Length of the interpolation.
    /// * `easing` - Curve applied to linear progress.
    /// * `started_at` - Timestamp of the first frame.
    ///
    /// # Returns
    /// A tween evaluated relative to `started_at`.
    #[inline]
    pub fn new_at(
        from: f64,
        to: f64,
        duration: Duration,
        easing: Easing,
        started_at: Instant,
    ) -> Self {
        Self {
            started_at,
            duration,
            from,
            to,
            easing,
        }
    }

    /// Create a tween that is already resting at `value`.
    #[inline]
    pub fn settled(value: f64) -> Self {
        Self::settled_at(value, Instant::now())
    }

    /// Create a tween that is already resting at `value` with an explicit timestamp.
    #[inline]
    pub fn settled_at(value: f64, started_at: Instant) -> Self {
        Self {
            started_at,
            duration: Duration::ZERO,
            from: value,
            to: value,
            easing: Easing::Linear,
        }
    }

    /// Animate from the current value to `to`.
    #[inline]
    pub fn retarget(&mut self, to: f64, duration: Duration, easing: Easing) {
        self.retarget_at(to, duration, easing, Instant::now());
    }

    /// Animate from the current value evaluated at `now` to `to`.
    ///
    /// # Arguments
    /// * `to` - New target value.
    /// * `duration` - Length of the new interpolation.
    /// * `easing` - Curve applied to linear progress.
    /// * `now` - Timestamp of the retarget event.
    #[inline]
    pub fn retarget_at(&mut self, to: f64, duration: Duration, easing: Easing, now: Instant) {
        let from = self.value_at(now);
        if from == to {
            *self = Self::settled_at(to, now);
        } else {
            *self = Self::new_at(from, to, duration, easing, now);
        }
    }

    /// Animate to `to` with [`DEFAULT_DURATION`] and the default easing.
    #[inline]
    pub fn animate_to(&mut self, to: f64) {
        self.retarget(to, DEFAULT_DURATION, Easing::default());
    }

    /// Animate to `to` with [`DEFAULT_DURATION`] and the default easing evaluated at `now`.
    #[inline]
    pub fn animate_to_at(&mut self, to: f64, now: Instant) {
        self.retarget_at(to, DEFAULT_DURATION, Easing::default(), now);
    }

    /// Restart with explicit endpoints, ignoring the current value.
    #[inline]
    pub fn restart(&mut self, from: f64, to: f64, duration: Duration, easing: Easing) {
        *self = Self::new(from, to, duration, easing);
    }

    /// Restart with explicit endpoints at specified time `now`.
    #[inline]
    pub fn restart_at(
        &mut self,
        from: f64,
        to: f64,
        duration: Duration,
        easing: Easing,
        now: Instant,
    ) {
        *self = Self::new_at(from, to, duration, easing, now);
    }

    /// Jump to `value` and stop any running animation.
    #[inline]
    pub fn snap_to(&mut self, value: f64) {
        *self = Self::settled(value);
    }

    /// Jump to `value` and stop any running animation at specified time `now`.
    #[inline]
    pub fn snap_to_at(&mut self, value: f64, now: Instant) {
        *self = Self::settled_at(value, now);
    }

    /// Interpolated value evaluated at `now`.
    ///
    /// # Arguments
    /// * `now` - Timestamp to evaluate.
    ///
    /// # Returns
    /// The eased value between the endpoints.
    #[inline]
    pub fn value_at(&self, now: Instant) -> f64 {
        let p = self.eased_progress_at(now);
        match true {
            _ if p >= 1.0 => self.to,
            _ if p <= 0.0 => self.from,
            _ => self.from + (self.to - self.from) * p,
        }
    }

    /// Linear progress in `0.0..=1.0` evaluated at `now`, before easing.
    ///
    /// # Arguments
    /// * `now` - Timestamp to evaluate.
    ///
    /// # Returns
    /// Clamped linear progress; `1.0` for a zero-duration tween.
    #[inline]
    pub fn raw_progress_at(&self, now: Instant) -> f64 {
        if self.duration.is_zero() {
            1.0
        } else {
            let elapsed = now.saturating_duration_since(self.started_at);
            if elapsed >= self.duration {
                1.0
            } else {
                (elapsed.as_secs_f64() / self.duration.as_secs_f64()).clamp(0.0, 1.0)
            }
        }
    }

    /// Eased progress in `0.0..=1.0` evaluated at `now`.
    ///
    /// # Arguments
    /// * `now` - Timestamp to evaluate.
    ///
    /// # Returns
    /// The easing curve applied to linear progress.
    #[inline]
    pub fn eased_progress_at(&self, now: Instant) -> f64 {
        self.easing.apply(self.raw_progress_at(now))
    }

    /// Whether the tween still changes value at `now`.
    #[inline]
    pub fn is_animating_at(&self, now: Instant) -> bool {
        self.raw_progress_at(now) < 1.0
    }

    /// Monotonic microsecond tick since animation started up to `now`.
    #[inline]
    pub fn frame_tick_at(&self, now: Instant) -> u64 {
        tick_between(self.started_at, now)
    }

    /// Frame index since this tween started, quantized to the shared 120 fps budget.
    #[inline]
    pub fn frame_index(&self) -> u64 {
        frame_index_since(self.started_at)
    }
}
