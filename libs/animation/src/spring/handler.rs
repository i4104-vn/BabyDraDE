//! Spring state and the analytic harmonic-oscillator calculation.

use std::time::Instant;

/// Configuration for physical spring dynamics.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpringConfig {
    /// Spring stiffness ($k$). Higher values snap back faster and harder.
    pub stiffness: f64,
    /// Damping coefficient ($c$). Higher values suppress oscillation and bouncing.
    pub damping: f64,
    /// Mass of the animated object ($m$). Default is `1.0`.
    pub mass: f64,
}

impl Default for SpringConfig {
    #[inline]
    fn default() -> Self {
        Self::SMOOTH
    }
}

impl SpringConfig {
    /// Critically damped spring ($\zeta \approx 1.0$), smooth with zero overshoot.
    pub const SMOOTH: Self = Self {
        stiffness: 170.0,
        damping: 26.0,
        mass: 1.0,
    };

    /// Bouncy spring with pleasant, playful overshoot ($\zeta \approx 0.45$).
    pub const BOUNCY: Self = Self {
        stiffness: 180.0,
        damping: 12.0,
        mass: 1.0,
    };

    /// Snappy, responsive spring with minimal oscillation.
    pub const SNAPPY: Self = Self {
        stiffness: 320.0,
        damping: 28.0,
        mass: 1.0,
    };

    /// Gentle, soft spring for non-intrusive UI elements.
    pub const GENTLE: Self = Self {
        stiffness: 120.0,
        damping: 15.0,
        mass: 1.0,
    };

    /// High-energy, expressive oscillation with multiple visible bounces.
    pub const WOBBLY: Self = Self {
        stiffness: 260.0,
        damping: 10.0,
        mass: 1.0,
    };

    /// Create a spring configuration with explicit physical properties.
    ///
    /// # Arguments
    /// * `stiffness` - Spring constant, clamped to a positive minimum.
    /// * `damping` - Damping coefficient, clamped to zero.
    /// * `mass` - Moving mass, clamped to a positive minimum.
    ///
    /// # Returns
    /// A sanitized configuration.
    pub fn new(stiffness: f64, damping: f64, mass: f64) -> Self {
        Self {
            stiffness: stiffness.max(0.01),
            damping: damping.max(0.0),
            mass: mass.max(0.001),
        }
    }

    /// Create a configuration from damping ratio $\zeta$ and natural frequency (Hz).
    ///
    /// # Arguments
    /// * `damping_ratio` - Ratio of damping to critical damping.
    /// * `frequency_hz` - Undamped natural frequency in hertz.
    ///
    /// # Returns
    /// The equivalent physical spring configuration.
    pub fn from_damping_ratio(damping_ratio: f64, frequency_hz: f64) -> Self {
        let mass = 1.0;
        let omega0 = 2.0 * std::f64::consts::PI * frequency_hz.max(0.01);
        let stiffness = mass * omega0 * omega0;
        let damping = 2.0 * damping_ratio.max(0.0) * (stiffness * mass).sqrt();
        Self {
            stiffness,
            damping,
            mass,
        }
    }

    /// Compute natural undamped angular frequency $\omega_0 = \sqrt{k / m}$.
    ///
    /// # Returns
    /// Natural angular frequency in radians per second.
    #[inline]
    pub fn natural_frequency(&self) -> f64 {
        (self.stiffness / self.mass).sqrt()
    }

    /// Compute damping ratio $\zeta = c / (2 \sqrt{k \cdot m})$.
    ///
    /// # Returns
    /// The ratio of damping to critical damping.
    #[inline]
    pub fn damping_ratio(&self) -> f64 {
        self.damping / (2.0 * (self.stiffness * self.mass).sqrt())
    }
}

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
