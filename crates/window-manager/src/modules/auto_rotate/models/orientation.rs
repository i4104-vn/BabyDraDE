//! Pure orientation maths: gravity samples, quarter turns and transform steps.

use std::time::Duration;

use smithay::utils::Transform;

/// Sampling interval used when a driver does not request its own.
pub(crate) const DEFAULT_POLL_INTERVAL: Duration = Duration::from_millis(500);
/// Number of consecutive identical readings required before rotating.
pub(crate) const REQUIRED_STABLE_READINGS: u8 = 3;
/// Gravity magnitude below which the device is considered laid flat.
const FLAT_THRESHOLD: f64 = 0.35;

/// A two-axis acceleration sample in units of Earth gravity.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Gravity {
    pub(crate) x: f64,
    pub(crate) y: f64,
}

/// One of four physical quarters around the panel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Orientation(u8);

impl Orientation {
    /// Maps a gravity sample to its dominant axis.
    ///
    /// # Arguments
    /// * `gravity` - Acceleration in g units.
    ///
    /// # Returns
    /// The quarter the panel is held in, or `None` when it is laid flat.
    pub(crate) fn from_gravity(gravity: Gravity) -> Option<Self> {
        let Gravity { x, y } = gravity;
        if x.hypot(y) < FLAT_THRESHOLD {
            return None;
        }
        if y.abs() >= x.abs() {
            Some(Self(if y >= 0.0 { 0 } else { 2 }))
        } else {
            Some(Self(if x >= 0.0 { 3 } else { 1 }))
        }
    }

    /// Number of clockwise quarter turns from `baseline` to this orientation.
    pub(crate) fn steps_from(self, baseline: Self) -> u8 {
        (self.0 + 4 - baseline.0) % 4
    }
}

/// Rotates a transform clockwise by `steps` quarter turns.
///
/// Both the plain and flipped families are handled, so a panel mounted
/// upside down keeps its mirror while rotating.
pub(crate) fn rotate(transform: Transform, steps: u8) -> Transform {
    let mut result = transform;
    for _ in 0..steps % 4 {
        result = match result {
            Transform::Normal => Transform::_90,
            Transform::_90 => Transform::_180,
            Transform::_180 => Transform::_270,
            Transform::_270 => Transform::Normal,
            Transform::Flipped => Transform::Flipped90,
            Transform::Flipped90 => Transform::Flipped180,
            Transform::Flipped180 => Transform::Flipped270,
            Transform::Flipped270 => Transform::Flipped,
        };
    }
    result
}

#[cfg(test)]
mod tests {
    use super::{rotate, Gravity, Orientation};
    use smithay::utils::Transform;

    #[test]
    fn gravity_maps_to_quarters_and_ignores_flat() {
        assert_eq!(
            Orientation::from_gravity(Gravity { x: 0.0, y: 1.0 }),
            Some(Orientation(0))
        );
        assert_eq!(
            Orientation::from_gravity(Gravity { x: 1.0, y: 0.0 }),
            Some(Orientation(3))
        );
        assert_eq!(
            Orientation::from_gravity(Gravity { x: 0.0, y: -1.0 }),
            Some(Orientation(2))
        );
        assert_eq!(
            Orientation::from_gravity(Gravity { x: -1.0, y: 0.0 }),
            Some(Orientation(1))
        );
        assert_eq!(Orientation::from_gravity(Gravity { x: 0.0, y: 0.0 }), None);
    }

    #[test]
    fn orientation_steps_wrap_around() {
        assert_eq!(Orientation(3).steps_from(Orientation(0)), 3);
        assert_eq!(Orientation(0).steps_from(Orientation(3)), 1);
    }

    #[test]
    fn rotate_cycles_through_all_eight_transforms() {
        assert_eq!(rotate(Transform::Normal, 1), Transform::_90);
        assert_eq!(rotate(Transform::Normal, 4), Transform::Normal);
        assert_eq!(rotate(Transform::Flipped90, 1), Transform::Flipped180);
        assert_eq!(rotate(Transform::Flipped270, 1), Transform::Flipped);
    }
}
