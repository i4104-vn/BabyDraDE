//! Evaluates tween state into the values consumed by renderers.

use std::time::{Duration, Instant};

use super::handler::Tween;
use crate::curves::Easing;

impl Tween {
    /// Current interpolated value.
    ///
    /// # Returns
    /// The value at the current instant.
    #[inline]
    pub fn value(&self) -> f64 {
        self.value_at(Instant::now())
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
    ///
    /// # Returns
    /// Linear progress at the current instant.
    #[inline]
    pub fn raw_progress(&self) -> f64 {
        self.raw_progress_at(Instant::now())
    }

    /// Eased progress in `0.0..=1.0`.
    ///
    /// # Returns
    /// Eased progress at the current instant.
    #[inline]
    pub fn eased_progress(&self) -> f64 {
        self.eased_progress_at(Instant::now())
    }

    /// Whether the tween still changes value on subsequent frames.
    #[inline]
    pub fn is_animating(&self) -> bool {
        self.is_animating_at(Instant::now())
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::constants::{DEFAULT_DURATION, FRAME_INTERVAL};

    #[test]
    fn frame_index_tracks_the_120fps_budget() {
        let tween = Tween::new_at(
            0.0,
            1.0,
            Duration::from_secs(1),
            Easing::Linear,
            Instant::now() - FRAME_INTERVAL * 4 + Duration::from_micros(1),
        );
        assert_eq!(tween.frame_index(), 3);
    }

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
        let tween = Tween::new_at(
            0.0,
            100.0,
            Duration::from_millis(1000),
            Easing::Linear,
            start,
        );

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
