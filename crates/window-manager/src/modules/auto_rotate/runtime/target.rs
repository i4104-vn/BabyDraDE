//! Applies computed rotations to the compositor's outputs.

use smithay::utils::Transform;

use crate::{
    app::state::CalloopData,
    shared::ports::{InteractionBuffer, RotationTarget},
};

impl RotationTarget for CalloopData {
    fn output_names(&self) -> Vec<String> {
        self.state
            .windows
            .space
            .outputs()
            .map(|output| output.name())
            .collect()
    }

    fn configured_transform(&self, name: &str) -> Option<Transform> {
        crate::shared::orientation::configured_transform(name)
    }

    fn current_transform(&self, name: &str) -> Option<Transform> {
        self.state
            .windows
            .space
            .outputs()
            .find(|output| output.name() == name)
            .map(|output| output.current_transform())
    }

    fn apply_transforms(&mut self, transforms: &[(String, Transform)], touch: Transform) {
        for output in self.state.windows.space.outputs() {
            if let Some((_, transform)) = transforms.iter().find(|(name, _)| *name == output.name())
            {
                output.change_current_state(None, Some(*transform), None, None);
                smithay::desktop::layer_map_for_output(output).arrange();
            }
        }
        self.state.set_touch_transform(Some(touch));
    }
}
