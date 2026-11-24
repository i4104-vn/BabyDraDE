//! Spring state and the analytic harmonic-oscillator calculation.

use std::time::Instant;

use common::SpringConfig;

/// Solves harmonic oscillator displacement and velocity analytically.
#[inline]
fn solve_spring(config: &SpringConfig, y0: f64, v0: f64, elapsed: f64) -> (f64, f64) {
    let k = config.stiffness;
    let m = config.mass;
    let c = config.damping;

    let omega0 = (k / m).sqrt();
    let zeta = c / (2.0 * (k * m).sqrt());

    match true {
        _ if zeta < 0.9999 => {
            // Underdamped (oscillatory)
            let omega_d = omega0 * (1.0 - zeta * zeta).sqrt();
            let c1 = y0;
            let c2 = (v0 + zeta * omega0 * y0) / omega_d;

            let decay = (-zeta * omega0 * elapsed).exp();
            let sin_dt = (omega_d * elapsed).sin();
            let cos_dt = (omega_d * elapsed).cos();

            let y = decay * (c1 * cos_dt + c2 * sin_dt);
            let v = decay
                * ((c2 * omega_d - c1 * zeta * omega0) * cos_dt
                    - (c1 * omega_d + c2 * zeta * omega0) * sin_dt);
            (y, v)
        }
        _ if zeta <= 1.0001 => {
            // Critically damped
            let c1 = y0;
            let c2 = v0 + omega0 * y0;
            let decay = (-omega0 * elapsed).exp();

            let y = decay * (c1 + c2 * elapsed);
            let v = decay * (c2 - omega0 * (c1 + c2 * elapsed));
            (y, v)
        }
        _ => {
            // Overdamped
            let omega_d = omega0 * (zeta * zeta - 1.0).sqrt();
            let r1 = -zeta * omega0 - omega_d;
            let r2 = -zeta * omega0 + omega_d;

            let c1 = (v0 - r2 * y0) / (r1 - r2);
            let c2 = y0 - c1;

            let e1 = (r1 * elapsed).exp();
            let e2 = (r2 * elapsed).exp();

            let y = c1 * e1 + c2 * e2;
            let v = c1 * r1 * e1 + c2 * r2 * e2;
            (y, v)
        }
    }
}

/// A physics-driven spring animation for fluid, organic motion.
#[derive(Clone, Copy, Debug)]
pub struct Spring {
    pub(super) started_at: Instant,
    pub(super) from: f64,
    pub(super) to: f64,
    initial_velocity: f64,
    pub(super) config: SpringConfig,
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
    ///
    /// # Arguments
    /// * `from` - Starting position.
    /// * `to` - Target position.
    /// * `initial_velocity` - Starting velocity in units per second.
    /// * `config` - Physical spring properties.
    ///
    /// # Returns
    /// A spring that starts animating immediately.
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
    ///
    /// # Arguments
    /// * `from` - Starting position.
    /// * `to` - Target position.
    /// * `initial_velocity` - Starting velocity in units per second.
    /// * `config` - Physical spring properties.
    /// * `started_at` - Timestamp of the first frame.
    ///
    /// # Returns
    /// A spring evaluated relative to `started_at`.
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
    ///
    /// # Arguments
    /// * `position_threshold` - Maximum distance considered settled.
    /// * `velocity_threshold` - Maximum velocity considered settled.
    ///
    /// # Returns
    /// The spring with clamped thresholds applied.
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
    ///
    /// # Arguments
    /// * `to` - New target position.
    /// * `now` - Timestamp of the retarget event.
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

    /// Evaluates the spring at `now` into a position, velocity and settled flag.
    pub(crate) fn solve_at(&self, now: Instant) -> (f64, f64, bool) {
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
}
