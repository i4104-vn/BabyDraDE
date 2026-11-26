//! Event-loop wiring for automatic rotation.

use smithay::{
    reexports::calloop::{timer::Timer, EventLoop},
    utils::Transform,
};

use crate::{
    app::state::CalloopData,
    shared::ports::{InteractionBuffer, RotationTarget},
};

use super::{
    controller::AutoRotate,
    driver::{DriverFactory, DriverSelection},
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
            }
        }
        self.state.set_touch_transform(Some(touch));
    }
}

/// Starts automatic rotation when it is enabled and a driver is available.
///
/// The configured transform of each output is the baseline the sensor rotates
/// from, so fixed orientation and auto-rotation can be used together.
///
/// # Arguments
/// * `event_loop` - Loop that owns the sampling timer.
pub(crate) fn spawn(event_loop: &mut EventLoop<'_, CalloopData>) {
    let config = services::config::load_display_config();
    if !config.auto_rotate.unwrap_or(true) {
        tracing::info!("auto-rotate disabled by configuration");
        return;
    }

    let selection = DriverSelection::from_config(config.auto_rotate_driver.as_deref());
    let manager = DriverFactory::with_builtins().create(selection);
    let mut controller = AutoRotate::new(manager);
    if controller.driver_name() == "none" {
        tracing::info!("auto-rotate: no accelerometer driver available on this machine");
        return;
    }
    tracing::info!(driver = controller.driver_name(), "auto-rotate enabled");

    if let Err(err) = event_loop.handle().insert_source(
        Timer::from_duration(controller.poll_interval()),
        move |_, _, data: &mut CalloopData| {
            let interval = controller.poll_interval();
            controller.poll(data);
            smithay::reexports::calloop::timer::TimeoutAction::ToDuration(interval)
        },
    ) {
        tracing::warn!("cannot start auto-rotate timer: {err}");
    }
}
