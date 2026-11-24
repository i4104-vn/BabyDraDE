//! Display transform matrix derivation.

use smithay::utils::Transform;

/// Affine matrix mapping the raw frame buffer into the output's logical
/// orientation. Coefficients come from smithay's `Transform::transform_point_in`.
pub(crate) fn transform_matrix(transform: Transform, w: i32, h: i32) -> (f64, f64, f64, f64, f64, f64) {
    let (w, h) = (w as f64, h as f64);
    match transform {
        Transform::Normal => (1.0, 0.0, 0.0, 1.0, 0.0, 0.0),
        Transform::_90 => (0.0, 1.0, -1.0, 0.0, h, 0.0),
        Transform::_180 => (-1.0, 0.0, 0.0, -1.0, w, h),
        Transform::_270 => (0.0, -1.0, 1.0, 0.0, 0.0, w),
        Transform::Flipped => (-1.0, 0.0, 0.0, 1.0, w, 0.0),
        Transform::Flipped90 => (0.0, 1.0, 1.0, 0.0, 0.0, 0.0),
        Transform::Flipped180 => (1.0, 0.0, 0.0, -1.0, 0.0, h),
        Transform::Flipped270 => (0.0, -1.0, -1.0, 0.0, h, w),
    }
}
