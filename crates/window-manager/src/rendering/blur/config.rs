use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::info;

/// Configuration parameters for the BabyDraDE Blur Engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlurParams {
    /// Global toggle to enable or disable blur rendering
    pub enabled: bool,
    /// Whether to blur all application windows by default
    pub blur_all: bool,
    /// Dual Kawase blur radius (higher = softer, frosted glass effect)
    pub radius: f32,
    /// Number of downsample/upsample passes (1 to 4, default 2)
    pub passes: usize,
    /// Window corner radius for SDF rounded clipping (default 14.0px)
    pub corner_radius: f32,
    /// Surface opacity for windows matching blur (0.0 to 1.0, default 0.68).
    /// A lower value is intentional: the backdrop is rendered behind the client
    /// surface, so an almost opaque client would make the blur imperceptible.
    pub window_opacity: f32,
    /// Acrylic tint color [R, G, B, A] (default neutral frosted tint [0.10, 0.12, 0.16, 0.18])
    pub tint: [f32; 4],
    /// Subtle dithering noise amount to eliminate 8-bit color banding (default 0.012)
    pub noise: f32,
    /// Backlight exposure / brightness adjustment (default 1.0)
    pub brightness: f32,
    /// Contrast adjustment (default 1.0)
    pub contrast: f32,
    /// List of target app IDs, process names, or window titles to blur
    pub app_rules: Vec<String>,
}

impl Default for BlurParams {
    fn default() -> Self {
        Self {
            enabled: true,
            blur_all: false,
            radius: 18.0,
            passes: 3,
            corner_radius: 14.0,
            window_opacity: 0.68,
            tint: [0.10, 0.12, 0.16, 0.18],
            noise: 0.012,
            brightness: 1.0,
            contrast: 1.0,
            app_rules: vec![
                // Popular terminal emulators
                "foot".to_string(),
                "alacritty".to_string(),
                "kitty".to_string(),
                "wezterm".to_string(),
                "org.gnome.Terminal".to_string(),
                "terminal".to_string(),
                // BabyDraDE UI widgets
                "babydra-panel".to_string(),
                "babydra-topbar".to_string(),
                // Editors & file managers
                "code".to_string(),
                "nautilus".to_string(),
                "thunar".to_string(),
            ],
        }
    }
}

impl BlurParams {
    /// Path to user configuration file: ~/.config/babydra/blur.json
    pub fn config_path() -> Option<PathBuf> {
        let home = std::env::var_os("HOME")?;
        Some(
            PathBuf::from(home)
                .join(".config")
                .join("babydra")
                .join("blur.json"),
        )
    }

    /// Load configuration from environment variables and JSON file
    pub fn load() -> Self {
        let mut params = Self::default();

        // 1. Try to load from ~/.config/babydra/blur.json
        if let Some(path) = Self::config_path() {
            if path.is_file() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(loaded) = serde_json::from_str::<BlurParams>(&content) {
                        info!("Loaded BabyDraDE blur configuration from {:?}", path);
                        params = loaded;
                    }
                }
            } else {
                // Write default configuration if not present
                let _ = Self::save(&params);
            }
        }

        // 2. Override with environment variables if specified
        if let Ok(val) = std::env::var("BABYDRADE_BLUR_ALL") {
            params.blur_all = val == "1" || val.eq_ignore_ascii_case("true");
        }

        if let Ok(val) = std::env::var("BABYDRADE_BLUR_RADIUS") {
            if let Ok(rad) = val.parse::<f32>() {
                params.radius = rad.clamp(1.0, 32.0);
            }
        }

        if let Ok(val) = std::env::var("BABYDRADE_BLUR_OPACITY") {
            if let Ok(op) = val.parse::<f32>() {
                params.window_opacity = op.clamp(0.1, 1.0);
            }
        }

        if let Ok(apps) = std::env::var("BABYDRADE_BLUR_APPS") {
            let custom_apps: Vec<String> = apps
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if !custom_apps.is_empty() {
                params.app_rules = custom_apps;
            }
        }

        params
    }

    /// Save configuration to ~/.config/babydra/blur.json
    pub fn save(&self) -> Result<(), std::io::Error> {
        if let Some(path) = Self::config_path() {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let json = serde_json::to_string_pretty(self)
                .map_err(|e| std::io::Error::other(e.to_string()))?;
            std::fs::write(path, json)?;
        }
        Ok(())
    }
}
