//! Shared animation timing presets consumed by the compositor and UI crates.

use std::time::Duration;

/// Frame rate the compositor's animation loops aim for.
pub const TARGET_FPS: u32 = 120;

/// Duration of one frame at [`TARGET_FPS`] (about 8.33 ms).
pub const FRAME_INTERVAL: Duration = Duration::from_nanos(1_000_000_000 / TARGET_FPS as u64);

/// Default duration for UI transitions.
pub const DEFAULT_DURATION: Duration = Duration::from_millis(420);

/// Smooth highlight/selection step, e.g. keyboard navigation in a dock.
pub const NAVIGATION_DURATION: Duration = Duration::from_millis(200);

/// Short glide when the highlight moves between icons (pointer hover).
pub const HOVER_DURATION: Duration = Duration::from_millis(140);

/// Duration of a wheel/touchpad scroll step.
pub const SCROLL_DURATION: Duration = Duration::from_millis(260);

/// Overlay fade-in duration.
pub const FADE_IN_DURATION: Duration = Duration::from_millis(320);

/// Overlay fade-out duration.
pub const FADE_OUT_DURATION: Duration = Duration::from_millis(260);

/// Wallpaper circular-reveal duration.
pub const REVEAL_DURATION: Duration = Duration::from_millis(900);

/// Window open duration.
pub const WINDOW_OPEN_DURATION: Duration = Duration::from_millis(280);
/// Window close duration.
pub const WINDOW_CLOSE_DURATION: Duration = Duration::from_millis(260);
/// Window minimize duration.
pub const WINDOW_MINIMIZE_DURATION: Duration = Duration::from_millis(280);
/// Window restore duration.
pub const WINDOW_RESTORE_DURATION: Duration = Duration::from_millis(300);
/// Window geometry transition duration.
pub const WINDOW_GEOMETRY_DURATION: Duration = Duration::from_millis(340);

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

/// Launcher search overlay: one timeline for the dimmer and staggered reveal.
pub const SEARCH_REVEAL_DURATION: Duration = Duration::from_millis(900);

/// Alt+Tab switcher stage pulse, dock highlight and preview/scroll steps.
pub const STAGE_ANIM_DURATION: Duration = Duration::from_millis(500);
/// Dock target highlight duration.
pub const DOCK_TARGET_ANIM_DURATION: Duration = Duration::from_millis(420);
/// Switcher preview step duration.
pub const SWITCHER_PREVIEW_DURATION: Duration = Duration::from_millis(180);
/// Switcher scroll step duration.
pub const SWITCHER_SCROLL_DURATION: Duration = Duration::from_millis(160);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_frame_rate_is_120_fps() {
        assert_eq!(TARGET_FPS, 120);
        assert!(FRAME_INTERVAL >= Duration::from_micros(8_333));
        assert!(FRAME_INTERVAL <= Duration::from_micros(8_334));
        assert!(DEFAULT_DURATION >= Duration::from_millis(100));
    }

    #[test]
    fn damping_ratio_presets_match_their_intent() {
        assert!((SpringConfig::SMOOTH.damping_ratio() - 1.0).abs() < 0.01);
        assert!(SpringConfig::BOUNCY.damping_ratio() < 0.5);
        assert!(SpringConfig::GENTLE.damping_ratio() > SpringConfig::BOUNCY.damping_ratio());
    }

    #[test]
    fn spring_config_clamps_non_physical_inputs() {
        let config = SpringConfig::new(-1.0, -2.0, 0.0);
        assert_eq!(config.stiffness, 0.01);
        assert_eq!(config.damping, 0.0);
        assert_eq!(config.mass, 0.001);

        let derived = SpringConfig::from_damping_ratio(0.7, 2.0);
        assert!((derived.damping_ratio() - 0.7).abs() < 1e-9);
        assert!((derived.natural_frequency() - 2.0 * std::f64::consts::PI * 2.0).abs() < 1e-9);
    }
}
