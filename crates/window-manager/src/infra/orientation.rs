//! Resolves fixed monitor orientation from `~/.babydra/configs/display.toml`.
//!
//! The compositor applies the configured rotation when it creates each output.
//! When auto-rotation is enabled the sensor rotates relative to that baseline,
//! so a sideways-mounted panel keeps working; with auto-rotation disabled the
//! configured rotation is the final one.

use services::config::{load_display_config, models::MonitorOrientation};
use smithay::utils::{Physical, Size, Transform};

/// Resolves the transform configured for a physical connector.
///
/// # Arguments
/// * `connector` - DRM connector name such as `eDP-1`.
///
/// # Returns
/// The configured transform, or `None` when no valid fixed rotation exists.
pub(crate) fn configured_transform(connector: &str) -> Option<Transform> {
    to_transform(load_display_config().orientation_for(connector))
}

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

/// Converts a configured orientation into smithay's transform type.
fn to_transform(orientation: Option<MonitorOrientation>) -> Option<Transform> {
    orientation.map(|orientation| match orientation {
        MonitorOrientation::Normal => Transform::Normal,
        MonitorOrientation::Rotate90 => Transform::_90,
        MonitorOrientation::Rotate180 => Transform::_180,
        MonitorOrientation::Rotate270 => Transform::_270,
        MonitorOrientation::Flipped => Transform::Flipped,
        MonitorOrientation::Flipped90 => Transform::Flipped90,
        MonitorOrientation::Flipped180 => Transform::Flipped180,
        MonitorOrientation::Flipped270 => Transform::Flipped270,
    })
}
