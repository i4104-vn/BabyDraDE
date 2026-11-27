//! Always-on accelerometer rotation, independent of the display config.

use std::{cell::RefCell, collections::HashMap, error::Error, rc::Rc};

use services::sensors::rotation::{
    DriverFactory, DriverSelection, RotationDriver, RotationTracker, RotationWorker,
};
use wayland_client::protocol::wl_output::Transform;

use super::outputs::{connect_outputs, OutputConnection};

/// Continuously rotates every output from the accelerometer.
struct RotationSession {
    driver: RotationWorker,
    tracker: RotationTracker,
    outputs: OutputConnection,
    baselines: HashMap<String, Transform>,
}

impl RotationSession {
    /// Opens a dedicated Wayland connection and captures output baselines.
    fn new(driver: RotationWorker) -> Result<Self, Box<dyn Error>> {
        let outputs = connect_outputs()?;
        let baselines = outputs.state.transforms.clone();
        Ok(Self {
            driver,
            tracker: RotationTracker::new(),
            outputs,
            baselines,
        })
    }

    /// Reads one sample and applies a rotation once it becomes stable.
    fn poll(&mut self) {
        let _ = self.outputs.queue.dispatch_pending(&mut self.outputs.state);
        let Some(gravity) = self.driver.read() else {
            return;
        };
        if self.outputs.state.heads.is_empty() {
            return;
        }
        let Some(steps) = self.tracker.update(gravity) else {
            return;
        };
        let serial = self.outputs.state.serial.unwrap_or(self.outputs.serial);

        let configuration = self
            .outputs
            .manager
            .create_configuration(serial, &self.outputs.qh, ());
        for (name, head) in &self.outputs.state.heads {
            let base = self
                .baselines
                .get(name)
                .copied()
                .unwrap_or(Transform::Normal);
            configuration
                .enable_head(head, &self.outputs.qh, ())
                .set_transform(rotate_wl(base, steps));
        }
        configuration.apply();
        let _ = self.outputs.connection.flush();
        tracing::info!("greeter: auto-rotated outputs by {steps} quarter turn(s)");
    }
}

/// Starts accelerometer rotation regardless of the display configuration.
pub(super) fn start_auto_rotate() {
    let manager = DriverFactory::with_builtins().create(DriverSelection::Auto);
    if manager.name() == "none" {
        tracing::info!("greeter: no accelerometer driver available; auto-rotate skipped");
        return;
    }
    // Sampling happens on a worker thread: a stalled proxy must never block
    // the login UI's main loop.
    let driver = RotationWorker::spawn(manager);
    let interval = driver.poll_interval();
    let session = match RotationSession::new(driver) {
        Ok(session) => Rc::new(RefCell::new(session)),
        Err(error) => {
            tracing::warn!("greeter: cannot start auto-rotate: {error}");
            return;
        }
    };
    tracing::info!("greeter: auto-rotate enabled");
    glib::timeout_add_local(interval, move || {
        if let Ok(mut session) = session.try_borrow_mut() {
            session.poll();
        }
        glib::ControlFlow::Continue
    });
}

/// Rotates a protocol transform clockwise by `steps` quarter turns.
pub(super) fn rotate_wl(transform: Transform, steps: u8) -> Transform {
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
            _ => result,
        };
    }
    result
}
