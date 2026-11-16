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
    pub fn new(stiffness: f64, damping: f64, mass: f64) -> Self {
        Self {
            stiffness: stiffness.max(0.01),
            damping: damping.max(0.0),
            mass: mass.max(0.001),
        }
    }

    /// Create a configuration from damping ratio $\zeta$ and natural frequency (Hz).
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
    #[inline]
    pub fn natural_frequency(&self) -> f64 {
        (self.stiffness / self.mass).sqrt()
    }

    /// Compute damping ratio $\zeta = c / (2 \sqrt{k \cdot m})$.
    #[inline]
    pub fn damping_ratio(&self) -> f64 {
        self.damping / (2.0 * (self.stiffness * self.mass).sqrt())
    }
}
