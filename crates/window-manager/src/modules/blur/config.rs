//! Runtime overrides for the shared compositor blur configuration.
//!
//! The persisted configuration lives in `~/.babydra/configs/blur.toml`.
//! This module only keeps command-line/environment compatibility local to the
//! compositor and does not create a second configuration file.

pub(crate) use super::models::BlurParams;

pub(crate) fn load() -> BlurParams {
    let mut params = services::config::load_blur_config();

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

#[cfg(test)]
pub(crate) fn save(params: &BlurParams) {
    services::config::save_blur_config(params);
}
