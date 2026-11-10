//! Compositor background blur effect configuration models.

use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

fn default_blur_radius() -> f32 {
    18.0
}

fn default_blur_passes() -> usize {
    3
}

fn default_blur_corner_radius() -> f32 {
    14.0
}

fn default_blur_window_opacity() -> f32 {
    0.68
}

fn default_blur_tint() -> [f32; 4] {
    [0.10, 0.12, 0.16, 0.18]
}

fn default_blur_noise() -> f32 {
    0.012
}

fn default_blur_brightness() -> f32 {
    1.0
}

fn default_blur_contrast() -> f32 {
    1.0
}

fn default_blur_app_rules() -> Vec<String> {
    vec![
        "foot".to_string(),
        "alacritty".to_string(),
        "kitty".to_string(),
        "wezterm".to_string(),
        "org.gnome.Terminal".to_string(),
        "terminal".to_string(),
        "babydra-panel".to_string(),
        "code".to_string(),
        "nautilus".to_string(),
        "thunar".to_string(),
    ]
}

/// Shared compositor dual-Kawase blur configuration persisted in `~/.babydra/babydra.conf` under `[blur]`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BlurConfig {
    /// Whether background blur is globally enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// When true, blurs every window regardless of matching `app_rules`.
    #[serde(default)]
    pub blur_all: bool,
    /// Kernel blur radius (strength).
    #[serde(default = "default_blur_radius")]
    pub radius: f32,
    /// Number of downsample/upsample blur passes.
    #[serde(default = "default_blur_passes")]
    pub passes: usize,
    /// Window rounded corner radius in pixels.
    #[serde(default = "default_blur_corner_radius")]
    pub corner_radius: f32,
    /// Window background base opacity factor (0.0 to 1.0).
    #[serde(default = "default_blur_window_opacity")]
    pub window_opacity: f32,
    /// RGBA tint overlay vector applied over the blurred region.
    #[serde(default = "default_blur_tint")]
    pub tint: [f32; 4],
    /// Film grain noise intensity factor.
    #[serde(default = "default_blur_noise")]
    pub noise: f32,
    /// Post-blur brightness multiplier.
    #[serde(default = "default_blur_brightness")]
    pub brightness: f32,
    /// Post-blur contrast multiplier.
    #[serde(default = "default_blur_contrast")]
    pub contrast: f32,
    /// List of application names/classes that receive background blur.
    #[serde(default = "default_blur_app_rules")]
    pub app_rules: Vec<String>,
}

impl Default for BlurConfig {
    /// Returns standard balanced blur settings suitable for modern GPUs.
    ///
    /// # Returns
    ///
    /// A new [`BlurConfig`] instance with 18.0 radius, 3 passes, and default terminal/file manager rules.
    fn default() -> Self {
        Self {
            enabled: true,
            blur_all: false,
            radius: default_blur_radius(),
            passes: default_blur_passes(),
            corner_radius: default_blur_corner_radius(),
            window_opacity: default_blur_window_opacity(),
            tint: default_blur_tint(),
            noise: default_blur_noise(),
            brightness: default_blur_brightness(),
            contrast: default_blur_contrast(),
            app_rules: default_blur_app_rules(),
        }
    }
}
