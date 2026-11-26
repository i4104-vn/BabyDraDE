//! Shared mapping between display configuration and compositor transforms.
//!
//! Both the backend (fixed rotation at output creation) and the auto-rotate
//! feature (baseline for sensor-driven rotation) resolve configured
//! orientations through this module, so the vocabulary stays identical.

use services::config::{load_display_config, models::MonitorOrientation};
use smithay::utils::Transform;

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

/// Converts a configured orientation into smithay's transform type.
///
/// # Arguments
/// * `orientation` - Parsed configuration value, if present.
///
/// # Returns
/// The matching transform, or `None` when the value is absent.
pub(crate) fn to_transform(orientation: Option<MonitorOrientation>) -> Option<Transform> {
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
