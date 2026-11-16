//! Frame-rate independent animation primitives shared by BabyDraDE.
//!
//! Every animation is driven by wall-clock time, so its result does not depend
//! on how often a renderer runs. The frame-rate constants describe the frame
//! budget that animation loops should use; they never cap the animation itself.
//!
//! ```
//! use animation::{Easing, Tween, DEFAULT_DURATION};
//!
//! let mut fade = Tween::settled(0.0);
//! fade.retarget(1.0, DEFAULT_DURATION, Easing::EaseOutCubic);
//!
//! // The value is derived from the clock, so a 60 Hz renderer, a 120 Hz
//! // renderer and a stuttering renderer all reach the same end state.
//! assert!((0.0..=1.0).contains(&fade.value()));
//! ```

use std::time::{Duration, Instant};

/// Frame rate the compositor's animation loops aim for.
pub const TARGET_FPS: u32 = 120;

/// Duration of one frame at [`TARGET_FPS`] (about 8.33 ms).
pub const FRAME_INTERVAL: Duration = Duration::from_nanos(1_000_000_000 / TARGET_FPS as u64);

/// Default duration for UI transitions.
pub const DEFAULT_DURATION: Duration = Duration::from_millis(420);

/// Easing curves used by [`Tween`].
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
    pub fn apply(self, progress: f64) -> f64 {
        let t = if progress.is_nan() {
            0.0
        } else {
            progress.clamp(0.0, 1.0)
        };

        match self {
            Self::Linear => t,
            Self::EaseInQuad => t * t,
            Self::EaseOutQuad => 1.0 - (1.0 - t) * (1.0 - t),
            Self::EaseInOutQuad => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
            Self::EaseInCubic => t * t * t,
            Self::EaseOutCubic => 1.0 - (1.0 - t).powi(3),
            Self::EaseInOutCubic => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }
        }
    }
}

/// A clock-driven interpolation between two `f64` values.
///
/// `Tween` is cheap to copy and intended to live next to UI state. Calling
/// [`Tween::value`] computes the current value from the wall clock, so it can
/// be sampled once per frame at any refresh rate.
#[derive(Clone, Copy, Debug)]
pub struct Tween {
    started_at: Instant,
    duration: Duration,
    from: f64,
    to: f64,
    easing: Easing,
}

impl Default for Tween {
    fn default() -> Self {
        Self::settled(0.0)
    }
}

impl Tween {
    /// Create a tween that starts immediately and animates `from -> to`.
    pub fn new(from: f64, to: f64, duration: Duration, easing: Easing) -> Self {
        Self {
            started_at: Instant::now(),
            duration,
            from,
            to,
            easing,
        }
    }

    /// Create a tween that is already resting at `value`.
    pub fn settled(value: f64) -> Self {
        Self {
            started_at: Instant::now(),
            duration: Duration::ZERO,
            from: value,
            to: value,
            easing: Easing::Linear,
        }
    }

    /// Animate from the current value to `to`.
    ///
    /// If the value is already at the target the tween settles immediately,
    /// which keeps render caches from waking up for a no-op animation.
    pub fn retarget(&mut self, to: f64, duration: Duration, easing: Easing) {
        let from = self.value();
        if from == to {
            *self = Self::settled(to);
        } else {
            *self = Self::new(from, to, duration, easing);
        }
    }

    /// Animate to `to` with [`DEFAULT_DURATION`] and the default easing.
    pub fn animate_to(&mut self, to: f64) {
        self.retarget(to, DEFAULT_DURATION, Easing::default());
    }

    /// Restart with explicit endpoints, ignoring the current value.
    ///
    /// Useful for one-shot pulses such as a preview that briefly scales down
    /// and back up on every selection change.
    pub fn restart(&mut self, from: f64, to: f64, duration: Duration, easing: Easing) {
        *self = Self::new(from, to, duration, easing);
    }

    /// Jump to `value` and stop any running animation.
    pub fn snap_to(&mut self, value: f64) {
        *self = Self::settled(value);
    }

    /// Current interpolated value.
    pub fn value(&self) -> f64 {
        self.from + (self.to - self.from) * self.eased_progress()
    }

    /// Value the tween is heading towards.
    pub fn target(&self) -> f64 {
        self.to
    }

    /// Value the tween started from.
    pub fn from(&self) -> f64 {
        self.from
    }

    /// Linear progress in `0.0..=1.0`, before easing.
    pub fn raw_progress(&self) -> f64 {
        if self.duration.is_zero() {
            1.0
        } else {
            (self.started_at.elapsed().as_secs_f64() / self.duration.as_secs_f64()).clamp(0.0, 1.0)
        }
    }

    /// Eased progress in `0.0..=1.0`.
    pub fn eased_progress(&self) -> f64 {
        self.easing.apply(self.raw_progress())
    }

    /// Whether the tween still changes value on subsequent frames.
    pub fn is_animating(&self) -> bool {
        self.raw_progress() < 1.0
    }

    /// Instant the current animation started.
    ///
    /// Together with [`Tween::frame_tick`] this forms a compact render-cache
    /// key: a new animation or a new frame invalidates cached output.
    pub fn started_at(&self) -> Instant {
        self.started_at
    }

    /// Monotonic microsecond tick since the animation started.
    ///
    /// Microsecond precision is intentionally finer than a 120 fps frame
    /// budget, so sampling at any refresh rate produces a distinct key.
    pub fn frame_tick(&self) -> u64 {
        tick_since(self.started_at)
    }
}

/// Monotonic microsecond tick since `started_at`.
///
/// Suitable as a render-cache key for anything animated by a clock rather
/// than a fixed 16 ms frame counter.
pub fn tick_since(started_at: Instant) -> u64 {
    started_at.elapsed().as_micros().min(u64::MAX as u128) as u64
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
    fn target_frame_rate_is_120_fps() {
        assert_eq!(TARGET_FPS, 120);
        // 8.33 ms per frame, comfortably below the old 16 ms budget.
        assert!(FRAME_INTERVAL >= Duration::from_micros(8_333));
        assert!(FRAME_INTERVAL <= Duration::from_micros(8_334));
        assert!(DEFAULT_DURATION >= Duration::from_millis(100));
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
    fn easing_clamps_out_of_range_and_nan_input() {
        assert_eq!(Easing::Linear.apply(-1.0), 0.0);
        assert_eq!(Easing::Linear.apply(2.0), 1.0);
        assert_eq!(Easing::Linear.apply(f64::NAN), 0.0);
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

        // Immediately after retargeting the value must not jump.
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

        // Giống cách switcher cộng dồn mỗi lần bấm Tab: các lần bấm liên tiếp
        // không được kéo đích lùi về giá trị cũ.
        assert_eq!(tween.target(), 4.0);
    }

    #[test]
    fn frame_tick_is_monotonic_and_microsecond_grained() {
        let started = Instant::now();
        let first = tick_since(started);
        let second = tick_since(started);

        assert!(second >= first);
    }
}
