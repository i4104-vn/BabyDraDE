//! Per-output rotation baselines.
//!
//! A baseline is the transform a panel had before the sensor rotated it. It is
//! resolved once per output name from the fixed configuration (or, when there
//! is none, from the transform observed at first sight) and reused afterwards,
//! so reading a previously rotated output never drifts the baseline.

use std::collections::HashMap;

use smithay::utils::Transform;

use crate::shared::ports::RotationTarget;

#[derive(Debug, Default)]
pub(crate) struct Baselines(HashMap<String, Transform>);

impl Baselines {
    /// Returns the baseline for an output, caching it on first use.
    ///
    /// # Arguments
    /// * `target` - State used to resolve configured and current transforms.
    /// * `name` - Output connector name.
    ///
    /// # Returns
    /// The configured transform, the first observed transform, or `Normal`.
    pub(crate) fn transform_for(&mut self, target: &impl RotationTarget, name: &str) -> Transform {
        *self.0.entry(name.to_owned()).or_insert_with(|| {
            target
                .configured_transform(name)
                .or_else(|| target.current_transform(name))
                .unwrap_or(Transform::Normal)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Baselines;
    use crate::shared::ports::RotationTarget;
    use smithay::utils::Transform;

    #[derive(Default)]
    struct Target {
        configured: Option<Transform>,
        current: Option<Transform>,
    }

    impl RotationTarget for Target {
        fn output_names(&self) -> Vec<String> {
            Vec::new()
        }
        fn configured_transform(&self, _name: &str) -> Option<Transform> {
            self.configured
        }
        fn current_transform(&self, _name: &str) -> Option<Transform> {
            self.current
        }
        fn apply_transforms(&mut self, _: &[(String, Transform)], _: Transform) {}
    }

    #[test]
    fn configured_transform_wins_over_current() {
        let target = Target {
            configured: Some(Transform::Flipped),
            current: Some(Transform::_90),
        };
        let mut baselines = Baselines::default();
        assert_eq!(
            baselines.transform_for(&target, "eDP-1"),
            Transform::Flipped
        );
    }

    #[test]
    fn cached_baseline_survives_a_changed_current_transform() {
        let target = Target {
            configured: None,
            current: Some(Transform::Normal),
        };
        let mut baselines = Baselines::default();
        baselines.transform_for(&target, "eDP-1");
        let rotated = Target {
            configured: None,
            current: Some(Transform::_90),
        };
        assert_eq!(
            baselines.transform_for(&rotated, "eDP-1"),
            Transform::Normal
        );
    }
}
