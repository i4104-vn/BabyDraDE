//! Automatic panel rotation from a device orientation sensor.
//!
//! Two sensor sources are supported, in order of preference:
//!   1. raw IIO sysfs accelerometers (`/sys/bus/iio/devices/iio:deviceN`);
//!   2. `iio-sensor-proxy` on D-Bus (`net.hadess.SensorProxy`).
//!
//! Rotation follows the *change* from the orientation seen at startup, so a
//! panel mounted sideways still rotates correctly. Machines with no sensor are
//! a no-op.

use gio::glib::Variant;
use smithay::{
    reexports::calloop::{
        timer::{TimeoutAction, Timer},
        EventLoop,
    },
    utils::Transform,
};
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::adapters::compositor::CalloopData;

const POLL_INTERVAL: Duration = Duration::from_millis(500);
const REQUIRED_STABLE_READINGS: u8 = 3;
const EARTH_GRAVITY: f64 = 9.81;

const PROXY_NAME: &str = "net.hadess.SensorProxy";
const PROXY_PATH: &str = "/net/hadess/SensorProxy";
const PROXY_IFACE: &str = "net.hadess.SensorProxy";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Orientation(u8);

impl Orientation {
    /// Dominant gravity axis: Y down, X right, Y up, X left.
    fn from_gravity(x: f64, y: f64) -> Option<Self> {
        let magnitude = x.hypot(y);
        if magnitude < 0.35 {
            return None; // laying flat: keep the current orientation
        }
        if y.abs() >= x.abs() {
            Some(Self(if y >= 0.0 { 0 } else { 2 }))
        } else {
            Some(Self(if x >= 0.0 { 3 } else { 1 }))
        }
    }

    fn steps_from(self, baseline: Self) -> u8 {
        (self.0 + 4 - baseline.0) % 4
    }
}

enum Sensor {
    Iio { path: PathBuf, scale: f64 },
    Proxy(gio::DBusConnection),
}

impl Sensor {
    fn read_gravity(&self) -> Option<(f64, f64)> {
        match self {
            Self::Iio { path, scale } => {
                let x = read_number(&path.join("in_accel_x_raw"))? * scale;
                let y = read_number(&path.join("in_accel_y_raw"))? * scale;
                Some((x, y))
            }
            Self::Proxy(connection) => {
                let reply = connection
                    .call_sync(
                        Some(PROXY_NAME),
                        PROXY_PATH,
                        "org.freedesktop.DBus.Properties",
                        "Get",
                        Some(&Variant::from((PROXY_IFACE, "Accelerate"))),
                        None,
                        gio::DBusCallFlags::NONE,
                        500,
                        gio::Cancellable::NONE,
                    )
                    .ok()?;
                let value = reply.child_value(0).get::<Variant>()?;
                let x = value.child_value(0).get::<f64>()? / EARTH_GRAVITY;
                let y = value.child_value(1).get::<f64>()? / EARTH_GRAVITY;
                Some((x, y))
            }
        }
    }
}

impl std::fmt::Debug for Sensor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Iio { path, .. } => write!(f, "IIO:{path:?}"),
            Self::Proxy(_) => write!(f, "iio-sensor-proxy"),
        }
    }
}

struct AutoRotate {
    sensor: Sensor,
    baseline: Option<Orientation>,
    base_transform: Option<Transform>,
    candidate: Option<Orientation>,
    stable: u8,
    applied: u8,
}

impl AutoRotate {
    fn new(sensor: Sensor) -> Self {
        Self {
            sensor,
            baseline: None,
            base_transform: None,
            candidate: None,
            stable: 0,
            applied: 0,
        }
    }

    fn poll(&mut self, state: &mut CalloopData) {
        let Some((x, y)) = self.sensor.read_gravity() else {
            return;
        };
        if std::env::var("BABYDRADE_AUTO_ROTATE_DEBUG").as_deref() == Ok("1") {
            tracing::debug!("accelerometer: x={x:.3} y={y:.3}");
        }
        let Some(current) = Orientation::from_gravity(x, y) else {
            return;
        };
        let baseline = *self.baseline.get_or_insert(current);
        let base_transform = *self
            .base_transform
            .get_or_insert_with(|| baseline_transform(state));

        if self.candidate == Some(current) {
            self.stable += 1;
        } else {
            self.candidate = Some(current);
            self.stable = 1;
        }
        if self.stable < REQUIRED_STABLE_READINGS || self.applied == current.steps_from(baseline) {
            return;
        }

        self.applied = current.steps_from(baseline);
        self.stable = 0;

        let transform = rotate(base_transform, self.applied);
        for output in state.state.space.outputs() {
            output.change_current_state(None, Some(transform), None, None);
        }
        std::env::set_var("BABYDRADE_TOUCH_TRANSFORM", transform_name(transform));
        tracing::info!("auto-rotated panel to {transform:?}");
    }
}


mod ops;

#[cfg(test)]
mod tests;

pub(crate) use ops::spawn;
use ops::{baseline_transform, read_number, rotate, transform_name};
