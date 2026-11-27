//! Event-loop wiring for automatic rotation.

use smithay::reexports::calloop::{timer::Timer, EventLoop};

use services::sensors::rotation::{DriverFactory, DriverSelection, RotationDriver, RotationWorker};

use crate::app::state::CalloopData;

use super::super::controller::AutoRotate;

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
    if manager.name() == "none" {
        tracing::info!("auto-rotate: no accelerometer driver available on this machine");
        return;
    }
    // Reading a sensor can block for seconds on a stalled proxy; keep it off
    // the event loop so input and rendering stay responsive.
    let mut controller = AutoRotate::new(RotationWorker::spawn(manager));
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
