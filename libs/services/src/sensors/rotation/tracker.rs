//! Pure orientation maths and the gravity debounce state machine.

/// Number of consecutive identical readings required before rotating.
pub const REQUIRED_STABLE_READINGS: u8 = 3;
/// Gravity magnitude below which the device is considered laid flat.
const FLAT_THRESHOLD: f64 = 0.35;

/// A two-axis acceleration sample in units of Earth gravity.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gravity {
    pub x: f64,
    pub y: f64,
}

/// One of four physical quarters around the panel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Orientation(u8);

impl Orientation {
    /// Maps a gravity sample to its dominant axis.
    ///
    /// # Arguments
    /// * `gravity` - Acceleration in g units.
    ///
    /// # Returns
    /// The quarter the panel is held in, or `None` when it is laid flat.
    pub fn from_gravity(gravity: Gravity) -> Option<Self> {
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
    pub fn steps_from(self, baseline: Self) -> u8 {
        (self.0 + 4 - baseline.0) % 4
    }
}

/// Debounces gravity samples relative to the orientation seen at startup.
#[derive(Default)]
pub struct RotationTracker {
    baseline: Option<Orientation>,
    candidate: Option<Orientation>,
    stable: u8,
    applied: u8,
}

impl RotationTracker {
    /// Creates a tracker anchored on the first valid sample.
    pub fn new() -> Self {
        Self::default()
    }

    /// Feeds one sample and reports a new rotation once it is stable.
    ///
    /// # Arguments
    /// * `gravity` - Latest acceleration in g units.
    ///
    /// # Returns
    /// The number of clockwise quarter turns to apply, or `None` while the
    /// reading is flat, unchanged or not yet stable.
    pub fn update(&mut self, gravity: Gravity) -> Option<u8> {
        let current = Orientation::from_gravity(gravity)?;
        let baseline = *self.baseline.get_or_insert(current);
        if self.candidate == Some(current) {
            self.stable += 1;
        } else {
            self.candidate = Some(current);
            self.stable = 1;
        }

        let steps = current.steps_from(baseline);
        if self.stable < REQUIRED_STABLE_READINGS || self.applied == steps {
            return None;
        }
        self.applied = steps;
        self.stable = 0;
        Some(steps)
    }
}

#[cfg(test)]
mod tests {
    use super::{Gravity, Orientation, RotationTracker};

    fn down() -> Gravity {
        Gravity { x: 0.0, y: 1.0 }
    }

    fn right() -> Gravity {
        Gravity { x: 1.0, y: 0.0 }
    }

    #[test]
    fn gravity_maps_to_quarters_and_ignores_flat() {
        assert_eq!(
            Orientation::from_gravity(down()),
            Some(Orientation(0))
        );
        assert_eq!(
            Orientation::from_gravity(right()),
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
    fn reports_a_change_only_after_three_stable_readings() {
        let mut tracker = RotationTracker::new();
        assert_eq!(tracker.update(down()), None);
        assert_eq!(tracker.update(right()), None);
        assert_eq!(tracker.update(right()), None);
        assert_eq!(tracker.update(right()), Some(3));
    }

    #[test]
    fn ignores_unstable_readings() {
        let mut tracker = RotationTracker::new();
        for gravity in [down(), right(), down(), right(), down()] {
            assert_eq!(tracker.update(gravity), None);
        }
    }
}
