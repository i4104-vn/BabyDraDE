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

pub(super) const POLL_INTERVAL: Duration = Duration::from_millis(500);
pub(super) const REQUIRED_STABLE_READINGS: u8 = 3;
pub(super) const EARTH_GRAVITY: f64 = 9.81;

pub(super) const PROXY_NAME: &str = "net.hadess.SensorProxy";
pub(super) const PROXY_PATH: &str = "/net/hadess/SensorProxy";
pub(super) const PROXY_IFACE: &str = "net.hadess.SensorProxy";

mod models;
mod handler;



#[cfg(test)]
mod tests;

pub(crate) use handler::spawn;
use handler::{baseline_transform, read_number, rotate, transform_name};
