//! Resolves fixed monitor orientation from `~/.babydra/configs/display.toml`.
//!
//! The compositor applies the configured rotation when it creates each output.
//! When auto-rotation is enabled the sensor rotates relative to that baseline,
//! so a sideways-mounted panel keeps working; with auto-rotation disabled the
//! configured rotation is the final one.

pub(crate) use crate::shared::orientation::configured_transform;

use crate::shared::orientation::to_transform;
use services::config::load_display_config;
use smithay::utils::{Physical, Size, Transform};

/// Resolves the transform configured for the single nested Winit output.
///
/// # Returns
/// The transform of the first enabled monitor, or `None` when unconfigured.
pub(crate) fn primary_transform() -> Option<Transform> {
    to_transform(load_display_config().primary_orientation())
}

/// Returns the transform used when a portrait panel has no fixed rotation.
///
/// # Arguments
/// * `size` - Physical mode size reported by the connector.
///
/// # Returns
/// A quarter turn for portrait panels, `Normal` otherwise.
pub(crate) fn landscape_fallback(size: Size<i32, Physical>) -> Transform {
    if size.w < size.h {
        Transform::_90
    } else {
        Transform::Normal
    }
}
