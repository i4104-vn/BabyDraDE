//! Evaluates spring state into the values consumed by renderers.

use std::time::Instant;

use super::handler::Spring;
use common::SpringConfig;

impl Spring {
    /// Current interpolated position.
    ///
    /// # Returns
    /// The position at the current instant.
    #[inline]
    pub fn value(&self) -> f64 {
        self.value_at(Instant::now())
    }

    /// Current velocity (units per second).
    ///
    /// # Returns
    /// The velocity at the current instant.
    #[inline]
    pub fn velocity(&self) -> f64 {
        self.velocity_at(Instant::now())
    }

    /// Whether the spring is still actively oscillating or moving.
    ///
    /// # Returns
    /// `true` while the spring has not settled.
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

    /// Evaluated position at `now`.
    ///
    /// # Arguments
    /// * `now` - Timestamp to evaluate.
    ///
    /// # Returns
    /// The interpolated position.
    #[inline]
    pub fn value_at(&self, now: Instant) -> f64 {
        let (val, _, _) = self.solve_at(now);
        val
    }

    /// Evaluated velocity at `now`.
    ///
    /// # Arguments
    /// * `now` - Timestamp to evaluate.
    ///
    /// # Returns
    /// The instantaneous velocity.
    #[inline]
    pub fn velocity_at(&self, now: Instant) -> f64 {
        let (_, vel, _) = self.solve_at(now);
        vel
    }

    /// Whether the spring is still animating at `now`.
    ///
    /// # Arguments
    /// * `now` - Timestamp to evaluate.
    ///
    /// # Returns
    /// `true` while motion continues.
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
