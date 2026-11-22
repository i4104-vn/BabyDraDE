pub mod config;
pub mod solver;

use std::time::Instant;

pub use config::SpringConfig;
use solver::solve_spring;

/// A physics-driven spring animation for fluid, organic motion.
#[derive(Clone, Copy, Debug)]
pub struct Spring {
    started_at: Instant,
    from: f64,
    to: f64,
    initial_velocity: f64,
    config: SpringConfig,
    settle_threshold: f64,
    settle_velocity_threshold: f64,
}

impl Default for Spring {
    #[inline]
    fn default() -> Self {
        Self::settled(0.0)
    }
}

impl Spring {
    /// Create a new spring animating `from -> to` starting from rest.
    #[inline]
    pub fn new(from: f64, to: f64, config: SpringConfig) -> Self {
        Self::new_with_velocity_at(from, to, 0.0, config, Instant::now())
    }

    /// Create a spring with initial velocity.
    #[inline]
    pub fn new_with_velocity(
        from: f64,
        to: f64,
        initial_velocity: f64,
        config: SpringConfig,
    ) -> Self {
        Self::new_with_velocity_at(from, to, initial_velocity, config, Instant::now())
    }

    /// Create a spring with an explicit start time and initial velocity.
    #[inline]
    pub fn new_with_velocity_at(
        from: f64,
        to: f64,
        initial_velocity: f64,
        config: SpringConfig,
        started_at: Instant,
    ) -> Self {
        Self {
            started_at,
            from,
            to,
            initial_velocity,
            config,
            settle_threshold: 0.001,
            settle_velocity_threshold: 0.01,
        }
    }

    /// Create a spring that is already resting at `value`.
    #[inline]
    pub fn settled(value: f64) -> Self {
        Self::settled_at(value, Instant::now())
    }

    /// Create a settled spring at a specific timestamp.
    #[inline]
    pub fn settled_at(value: f64, started_at: Instant) -> Self {
        Self {
            started_at,
            from: value,
            to: value,
            initial_velocity: 0.0,
            config: SpringConfig::default(),
            settle_threshold: 0.001,
            settle_velocity_threshold: 0.01,
        }
    }

    /// Adjust precision thresholds for settling detection.
    #[inline]
    pub fn with_settle_threshold(
        mut self,
        position_threshold: f64,
        velocity_threshold: f64,
    ) -> Self {
        self.settle_threshold = position_threshold.max(1e-6);
        self.settle_velocity_threshold = velocity_threshold.max(1e-6);
        self
    }

    /// Redirect spring towards a new target, seamlessly transferring current velocity.
    #[inline]
    pub fn retarget(&mut self, to: f64) {
        self.retarget_at(to, Instant::now());
    }

    /// Redirect spring towards a new target at a specific timestamp `now`.
    #[inline]
    pub fn retarget_at(&mut self, to: f64, now: Instant) {
        let current_value = self.value_at(now);
        let current_velocity = self.velocity_at(now);

        if (current_value - to).abs() <= self.settle_threshold
            && current_velocity.abs() <= self.settle_velocity_threshold
        {
            *self = Self::settled_at(to, now);
        } else {
            *self =
                Self::new_with_velocity_at(current_value, to, current_velocity, self.config, now);
        }
    }

    /// Instant jump to `value`, dropping all velocity.
    #[inline]
    pub fn snap_to(&mut self, value: f64) {
        *self = Self::settled(value);
    }

    /// Current interpolated position.
    #[inline]
    pub fn value(&self) -> f64 {
        self.value_at(Instant::now())
    }

    /// Current velocity (units per second).
    #[inline]
    pub fn velocity(&self) -> f64 {
        self.velocity_at(Instant::now())
    }

    /// Whether the spring is still actively oscillating or moving.
    #[inline]
    pub fn is_animating(&self) -> bool {
        self.is_animating_at(Instant::now())
    }

    /// Target position the spring pulls towards.
    #[inline]
    pub fn target(&self) -> f64 {
        self.to
    }

    /// Position the spring started from.
    #[inline]
    pub fn from(&self) -> f64 {
        self.from
    }

    /// Configuration used by this spring.
    #[inline]
    pub fn config(&self) -> SpringConfig {
        self.config
    }

    /// Start timestamp of current spring phase.
    #[inline]
    pub fn started_at(&self) -> Instant {
        self.started_at
    }

    fn solve_at(&self, now: Instant) -> (f64, f64, bool) {
        let elapsed = now.saturating_duration_since(self.started_at).as_secs_f64();
        let y0 = self.from - self.to;
        let v0 = self.initial_velocity;

        if y0.abs() <= self.settle_threshold && v0.abs() <= self.settle_velocity_threshold {
            return (self.to, 0.0, true);
        }

        if elapsed <= 0.0 {
            return (self.from, v0, false);
        }

        let (y, v) = solve_spring(&self.config, y0, v0, elapsed);

        let is_settled =
            y.abs() <= self.settle_threshold && v.abs() <= self.settle_velocity_threshold;
        if is_settled {
            (self.to, 0.0, true)
        } else {
            (self.to + y, v, false)
        }
    }

    /// Evaluated position at `now`.
    #[inline]
    pub fn value_at(&self, now: Instant) -> f64 {
        let (val, _, _) = self.solve_at(now);
        val
    }

    /// Evaluated velocity at `now`.
    #[inline]
    pub fn velocity_at(&self, now: Instant) -> f64 {
        let (_, vel, _) = self.solve_at(now);
        vel
    }

    /// Whether the spring is still animating at `now`.
    #[inline]
    pub fn is_animating_at(&self, now: Instant) -> bool {
        let (_, _, settled) = self.solve_at(now);
        !settled
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn settled_spring_returns_target() {
        let spring = Spring::settled(42.0);
        assert_eq!(spring.value(), 42.0);
        assert_eq!(spring.velocity(), 0.0);
        assert!(!spring.is_animating());
    }

    #[test]
    fn critically_damped_reaches_target_smoothly() {
        let start = Instant::now();
        let spring = Spring::new_with_velocity_at(0.0, 100.0, 0.0, SpringConfig::SMOOTH, start);

        assert_eq!(spring.value_at(start), 0.0);
        assert!(spring.is_animating_at(start));

        let mid = start + Duration::from_millis(200);
        assert!(spring.value_at(mid) > 50.0);
        assert!(spring.value_at(mid) < 100.0);

        let end = start + Duration::from_millis(1500);
        assert_eq!(spring.value_at(end), 100.0);
        assert!(!spring.is_animating_at(end));
    }

    #[test]
    fn bouncy_spring_overshoots_target() {
        let start = Instant::now();
        let spring = Spring::new_with_velocity_at(0.0, 100.0, 0.0, SpringConfig::BOUNCY, start);

        let mut overshot = false;
        for ms in (50..1000).step_by(10) {
            let t = start + Duration::from_millis(ms);
            if spring.value_at(t) > 100.0 {
                overshot = true;
                break;
            }
        }
        assert!(overshot, "BOUNCY spring should physically overshoot 100.0");
    }

    #[test]
    fn momentum_is_preserved_on_retarget() {
        let start = Instant::now();
        let mut spring = Spring::new_with_velocity_at(0.0, 100.0, 0.0, SpringConfig::SMOOTH, start);

        let mid = start + Duration::from_millis(100);
        let vel_before = spring.velocity_at(mid);
        assert!(vel_before > 0.0);

        spring.retarget_at(0.0, mid);
        assert_eq!(spring.velocity_at(mid), vel_before);
    }
}
