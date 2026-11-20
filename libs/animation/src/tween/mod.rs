use std::time::{Duration, Instant};

use crate::curves::Easing;
use crate::time::clock::tick_between;
use crate::time::constants::DEFAULT_DURATION;

/// A clock-driven interpolation between two `f64` values.
#[derive(Clone, Copy, Debug)]
pub struct Tween {
    started_at: Instant,
    duration: Duration,
    from: f64,
    to: f64,
    easing: Easing,
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

    /// Current interpolated value.
    #[inline]
    pub fn value(&self) -> f64 {
        self.value_at(Instant::now())
    }

    /// Interpolated value evaluated at `now`.
    #[inline]
    pub fn value_at(&self, now: Instant) -> f64 {
        let p = self.eased_progress_at(now);
        match true {
            _ if p >= 1.0 => self.to,
            _ if p <= 0.0 => self.from,
            _ => self.from + (self.to - self.from) * p,
        }
    }

    /// Value the tween is heading towards.
    #[inline]
    pub fn target(&self) -> f64 {
        self.to
    }

    /// Value the tween started from.
    #[inline]
    pub fn from(&self) -> f64 {
        self.from
    }

    /// Linear progress in `0.0..=1.0`, before easing.
    #[inline]
    pub fn raw_progress(&self) -> f64 {
        self.raw_progress_at(Instant::now())
    }

    /// Linear progress in `0.0..=1.0` evaluated at `now`, before easing.
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

    /// Eased progress in `0.0..=1.0`.
    #[inline]
    pub fn eased_progress(&self) -> f64 {
        self.eased_progress_at(Instant::now())
    }

    /// Eased progress in `0.0..=1.0` evaluated at `now`.
    #[inline]
    pub fn eased_progress_at(&self, now: Instant) -> f64 {
        self.easing.apply(self.raw_progress_at(now))
    }

    /// Whether the tween still changes value on subsequent frames.
    #[inline]
    pub fn is_animating(&self) -> bool {
        self.is_animating_at(Instant::now())
    }

    /// Whether the tween still changes value at `now`.
    #[inline]
    pub fn is_animating_at(&self, now: Instant) -> bool {
        self.raw_progress_at(now) < 1.0
    }

    /// Instant the current animation started.
    #[inline]
    pub fn started_at(&self) -> Instant {
        self.started_at
    }

    /// Duration of the current animation.
    #[inline]
    pub fn duration(&self) -> Duration {
        self.duration
    }

    /// Easing curve used for the animation.
    #[inline]
    pub fn easing(&self) -> Easing {
        self.easing
    }

    /// Monotonic microsecond tick since the animation started.
    #[inline]
    pub fn frame_tick(&self) -> u64 {
        self.frame_tick_at(Instant::now())
    }

    /// Monotonic microsecond tick since animation started up to `now`.
    #[inline]
    pub fn frame_tick_at(&self, now: Instant) -> u64 {
        tick_between(self.started_at, now)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settled_tween_rests_at_value() {
        let tween = Tween::settled(3.5);

        assert_eq!(tween.value(), 3.5);
        assert!(!tween.is_animating());
        assert_eq!(tween.raw_progress(), 1.0);
    }

    #[test]
    fn zero_duration_tween_is_immediately_at_target() {
        let tween = Tween::new(0.0, 10.0, Duration::ZERO, Easing::EaseInOutCubic);

        assert_eq!(tween.value(), 10.0);
        assert!(!tween.is_animating());
    }

    #[test]
    fn retarget_starts_from_the_current_value() {
        let mut tween = Tween::new(0.0, 10.0, Duration::from_secs(60), Easing::Linear);
        let before = tween.value();
        tween.retarget(20.0, Duration::from_secs(60), Easing::Linear);

        assert!((tween.value() - before).abs() < 1e-6);
        assert!((tween.from() - before).abs() < 1e-6);
        assert_eq!(tween.target(), 20.0);
        assert!(tween.is_animating());
    }

    #[test]
    fn retarget_to_the_same_value_settles_without_animating() {
        let mut tween = Tween::settled(4.0);
        tween.retarget(4.0, Duration::from_secs(60), Easing::Linear);

        assert!(!tween.is_animating());
        assert_eq!(tween.value(), 4.0);
    }

    #[test]
    fn repeated_retargets_accumulate_from_the_previous_target() {
        let mut tween = Tween::settled(2.0);
        tween.retarget(
            tween.target() + 1.0,
            DEFAULT_DURATION,
            Easing::EaseInOutCubic,
        );
        tween.retarget(
            tween.target() + 1.0,
            DEFAULT_DURATION,
            Easing::EaseInOutCubic,
        );

        assert_eq!(tween.target(), 4.0);
    }

    #[test]
    fn evaluated_at_instant_matches_expectations() {
        let start = Instant::now();
        let tween = Tween::new_at(0.0, 100.0, Duration::from_millis(1000), Easing::Linear, start);

        let half_time = start + Duration::from_millis(500);
        assert_eq!(tween.raw_progress_at(half_time), 0.5);
        assert_eq!(tween.value_at(half_time), 50.0);
        assert!(tween.is_animating_at(half_time));

        let end_time = start + Duration::from_millis(1000);
        assert_eq!(tween.raw_progress_at(end_time), 1.0);
        assert_eq!(tween.value_at(end_time), 100.0);
        assert!(!tween.is_animating_at(end_time));
    }

    #[test]
    fn value_boundary_precision_avoids_drift() {
        let start = Instant::now();
        let tween = Tween::new_at(0.1, 0.7, Duration::from_millis(100), Easing::Linear, start);
        let end_time = start + Duration::from_millis(100);
        assert_eq!(tween.value_at(end_time), 0.7);
    }
}
