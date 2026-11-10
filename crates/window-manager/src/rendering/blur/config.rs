//! Runtime overrides for the shared compositor blur configuration.
//!
//! The persisted configuration lives in `services::models::BabyDraConfig`.
//! This module only keeps command-line/environment compatibility local to the
//! compositor and does not create a second configuration file.

use services::config as shared_config;
use services::models::BlurConfig;

pub type BlurParams = BlurConfig;

pub(crate) fn load() -> BlurParams {
    let mut params = shared_config::load_blur_config();

    if let Ok(val) = std::env::var("BABYDRADE_BLUR_ENABLED") {
        params.enabled = val == "1" || val.eq_ignore_ascii_case("true");
    }

    if let Ok(val) = std::env::var("BABYDRADE_BLUR_ALL") {
        params.blur_all = val == "1" || val.eq_ignore_ascii_case("true");
    }

    if let Ok(val) = std::env::var("BABYDRADE_BLUR_RADIUS") {
        if let Ok(radius) = val.parse::<f32>() {
            params.radius = radius.clamp(1.0, 32.0);
        }
    }

    if let Ok(val) = std::env::var("BABYDRADE_BLUR_OPACITY") {
        if let Ok(opacity) = val.parse::<f32>() {
            params.window_opacity = opacity.clamp(0.1, 1.0);
        }
    }

    if let Ok(apps) = std::env::var("BABYDRADE_BLUR_APPS") {
        params.app_rules = apps
            .split(',')
            .map(str::trim)
            .filter(|app| !app.is_empty())
            .map(ToOwned::to_owned)
            .collect();
    }

    params
}

pub(crate) fn save(params: &BlurParams) {
    shared_config::save_blur_config(params);
}
