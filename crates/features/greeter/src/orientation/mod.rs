//! Applies monitor orientations to the greetd/cage session.
//!
//! Fixed rotations come from the display config on disk, while automatic
//! rotation from the accelerometer is always enabled for the greeter and does
//! not consult that config. Both are sent to cage over `zwlr_output_manager_v1`;
//! no external tool such as `wlr-randr` is needed.

use std::time::Duration;

mod auto;
mod config;
mod outputs;

#[cfg(test)]
mod tests;

/// Upper bound for each protocol round-trip, so a stalled compositor cannot
/// keep the login screen from appearing.
pub(super) const PROTOCOL_TIMEOUT: Duration = Duration::from_secs(2);

/// Applies every configured monitor rotation to the current Wayland session.
///
/// Errors are logged and ignored: orientation must never block the login UI.
pub(crate) fn apply() {
    let Some(path) = config::find_config() else {
        tracing::warn!("greeter: no readable display.toml found; keeping default orientation");
        return;
    };
    let display_config = config::load_config(&path);
    match outputs::apply_protocol(&display_config) {
        Ok(applied) => {
            tracing::info!("greeter: applied orientation to {applied} output(s)");
        }
        Err(error) => {
            tracing::warn!("greeter: cannot set orientation over Wayland: {error}");
        }
    }
    // The login screen must follow the sensor even when `auto_rotate` is
    // disabled in the config, so this is unconditional and config-independent.
    auto::start_auto_rotate();
}
