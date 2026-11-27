//! Rotation controller: reads the shared sensor service and applies transforms.
//!
//! The controller is independent of the compositor state and of any concrete
//! driver. It reads through [`RotationDriver`] and writes through
//! [`RotationTarget`], which keeps the physics and business rules testable.

use std::time::Duration;

use services::sensors::rotation::{RotationDriver, RotationTracker};
use smithay::utils::Transform;

use crate::shared::ports::RotationTarget;

use super::models::{rotate, Baselines};

pub(crate) struct AutoRotate<R: RotationDriver> {
    source: R,
    tracker: RotationTracker,
    baselines: Baselines,
}

impl<R: RotationDriver> AutoRotate<R> {
    /// Creates a controller around a driver source.
    pub(crate) fn new(source: R) -> Self {
        Self {
            source,
            tracker: RotationTracker::new(),
            baselines: Baselines::default(),
        }
    }

    /// Name of the driver currently producing readings.
    pub(crate) fn driver_name(&self) -> &str {
        self.source.name()
    }

    /// Interval at which this controller should be polled.
    pub(crate) fn poll_interval(&self) -> Duration {
        self.source.poll_interval()
    }

    /// Reads the sensor once and applies a rotation when the reading is stable.
    ///
    /// # Arguments
    /// * `target` - Output state the transforms are applied to.
    pub(crate) fn poll(&mut self, target: &mut impl RotationTarget) {
        let Some(gravity) = self.source.read() else {
            return;
        };
        let names = target.output_names();
        if names.is_empty() {
            return;
        }
        let Some(steps) = self.tracker.update(gravity) else {
            return;
        };

        let transforms: Vec<(String, Transform)> = names
            .iter()
            .map(|name| {
                let base = self.baselines.transform_for(target, name);
                (name.clone(), rotate(base, steps))
            })
            .collect();
        let touch = transforms[0].1;
        target.apply_transforms(&transforms, touch);
        tracing::info!(driver = self.driver_name(), ?touch, "auto-rotated panel");
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::time::Duration;

    use super::{AutoRotate, RotationDriver, RotationTarget, Transform};
    use services::sensors::rotation::Gravity;

    #[derive(Debug)]
    struct Source {
        readings: Vec<Option<Gravity>>,
        index: usize,
    }

    impl RotationDriver for Source {
        fn name(&self) -> &str {
            "fake"
        }
        fn read(&mut self) -> Option<Gravity> {
            let reading = self.readings.get(self.index).cloned().unwrap_or(None);
            self.index += 1;
            reading
        }
        fn poll_interval(&self) -> Duration {
            Duration::from_millis(1)
        }
    }

    #[derive(Default)]
    struct Target {
        names: Vec<String>,
        configured: HashMap<String, Transform>,
        current: HashMap<String, Transform>,
        applied: Vec<(String, Transform)>,
        touch: Option<Transform>,
    }

    impl RotationTarget for Target {
        fn output_names(&self) -> Vec<String> {
            self.names.clone()
        }
        fn configured_transform(&self, name: &str) -> Option<Transform> {
            self.configured.get(name).copied()
        }
        fn current_transform(&self, name: &str) -> Option<Transform> {
            self.current.get(name).copied()
        }
        fn apply_transforms(&mut self, transforms: &[(String, Transform)], touch: Transform) {
            self.applied = transforms.to_vec();
            self.touch = Some(touch);
        }
    }

    fn down() -> Option<Gravity> {
        Some(Gravity { x: 0.0, y: 1.0 })
    }

    fn right() -> Option<Gravity> {
        Some(Gravity { x: 1.0, y: 0.0 })
    }

    fn left() -> Option<Gravity> {
        Some(Gravity { x: -1.0, y: 0.0 })
    }

    fn controller(readings: Vec<Option<Gravity>>) -> AutoRotate<Source> {
        AutoRotate::new(Source { readings, index: 0 })
    }

    fn single_output() -> Target {
        Target {
            names: vec!["eDP-1".into()],
            current: HashMap::from([("eDP-1".into(), Transform::Normal)]),
            ..Target::default()
        }
    }

    #[test]
    fn applies_rotation_after_three_stable_readings() {
        let mut controller = controller(vec![down(), right(), right(), right()]);
        let mut target = single_output();
        for _ in 0..4 {
            controller.poll(&mut target);
        }
        assert_eq!(target.applied, vec![("eDP-1".to_string(), Transform::_270)]);
    }

    #[test]
    fn keeps_a_separate_baseline_for_each_output() {
        let mut controller = controller(vec![down(), right(), right(), right()]);
        let mut target = Target {
            names: vec!["eDP-1".into(), "HDMI-A-1".into()],
            configured: HashMap::from([("HDMI-A-1".into(), Transform::Flipped)]),
            ..single_output()
        };
        for _ in 0..4 {
            controller.poll(&mut target);
        }
        assert_eq!(
            target.applied,
            vec![
                ("eDP-1".to_string(), Transform::_270),
                ("HDMI-A-1".to_string(), Transform::Flipped270),
            ]
        );
    }

    #[test]
    fn does_not_rotate_while_readings_keep_changing() {
        let mut controller = controller(vec![down(), right(), left(), right(), left(), right()]);
        let mut target = single_output();
        for _ in 0..6 {
            controller.poll(&mut target);
        }
        assert!(target.applied.is_empty());
    }

    #[test]
    fn ignores_transient_sensor_failures() {
        let mut controller =
            controller(vec![down(), None, None, down(), right(), right(), right()]);
        let mut target = single_output();
        for _ in 0..7 {
            controller.poll(&mut target);
        }
        assert_eq!(target.applied, vec![("eDP-1".to_string(), Transform::_270)]);
    }

    #[test]
    fn does_not_apply_without_attached_outputs() {
        let mut controller = controller(vec![down(), right(), right(), right()]);
        let mut target = Target::default();
        for _ in 0..4 {
            controller.poll(&mut target);
        }
        assert!(target.applied.is_empty());
    }
}
