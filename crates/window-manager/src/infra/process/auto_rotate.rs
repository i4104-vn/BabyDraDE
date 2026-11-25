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

use crate::app::state::CalloopData;

pub(super) const POLL_INTERVAL: Duration = Duration::from_millis(500);
pub(super) const REQUIRED_STABLE_READINGS: u8 = 3;
pub(super) const EARTH_GRAVITY: f64 = 9.81;

pub(super) const PROXY_NAME: &str = "net.hadess.SensorProxy";
pub(super) const PROXY_PATH: &str = "/net/hadess/SensorProxy";
pub(super) const PROXY_IFACE: &str = "net.hadess.SensorProxy";

// Orientation sensor state models.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Orientation(pub(super) u8);

impl Orientation {
    /// Dominant gravity axis: Y down, X right, Y up, X left.
    pub(super) fn from_gravity(x: f64, y: f64) -> Option<Self> {
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

    pub(super) fn steps_from(self, baseline: Self) -> u8 {
        (self.0 + 4 - baseline.0) % 4
    }
}

pub(crate) enum Sensor {
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

pub(crate) struct AutoRotate {
    pub(super) sensor: Sensor,
    baseline: Option<Orientation>,
    base_transform: Option<Transform>,
    candidate: Option<Orientation>,
    stable: u8,
    applied: u8,
}

impl AutoRotate {
    pub(super) fn new(sensor: Sensor) -> Self {
        Self {
            sensor,
            baseline: None,
            base_transform: None,
            candidate: None,
            stable: 0,
            applied: 0,
        }
    }

    pub(super) fn poll(&mut self, state: &mut CalloopData) {
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
        for output in state.state.windows.space.outputs() {
            output.change_current_state(None, Some(transform), None, None);
        }
        std::env::set_var("BABYDRADE_TOUCH_TRANSFORM", transform_name(transform));
        tracing::info!("auto-rotated panel to {transform:?}");
    }
}

// Sensor discovery and transform calculation helpers.

pub(super) fn baseline_transform(data: &CalloopData) -> Transform {
    data.state
        .windows
        .space
        .outputs()
        .next()
        .map(|output| output.current_transform())
        .unwrap_or(Transform::Normal)
}

/// Rotates a transform clockwise by `steps` quarter turns.
pub(super) fn rotate(transform: Transform, steps: u8) -> Transform {
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

pub(super) fn transform_name(transform: Transform) -> &'static str {
    match transform {
        Transform::Normal => "normal",
        Transform::_90 => "90",
        Transform::_180 => "180",
        Transform::_270 => "270",
        Transform::Flipped => "flipped",
        Transform::Flipped90 => "flipped-90",
        Transform::Flipped180 => "flipped-180",
        Transform::Flipped270 => "flipped-270",
    }
}

pub(super) fn read_number(path: &Path) -> Option<f64> {
    std::fs::read_to_string(path)
        .ok()?
        .trim()
        .parse::<f64>()
        .ok()
}

fn find_iio_accelerometer() -> Option<Sensor> {
    for root in ["/sys/bus/iio/devices", "/sys/class/iio"] {
        let Ok(entries) = std::fs::read_dir(root) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.join("in_accel_x_raw").is_file() && path.join("in_accel_y_raw").is_file() {
                let scale = read_number(&path.join("in_accel_scale")).unwrap_or(1.0);
                return Some(Sensor::Iio { path, scale });
            }
        }
    }
    None
}

fn sensor_proxy() -> Option<Sensor> {
    let connection = gio::bus_get_sync(gio::BusType::Session, gio::Cancellable::NONE).ok()?;
    // Calling the proxy directly also activates it over D-Bus; `NameHasOwner`
    // would report "false" for an installed-but-not-yet-started service.
    if connection
        .call_sync(
            Some(PROXY_NAME),
            PROXY_PATH,
            PROXY_IFACE,
            "ClaimAccelerometer",
            None,
            None,
            gio::DBusCallFlags::NONE,
            1000,
            gio::Cancellable::NONE,
        )
        .is_err()
    {
        return None;
    }

    let available = connection
        .call_sync(
            Some(PROXY_NAME),
            PROXY_PATH,
            "org.freedesktop.DBus.Properties",
            "Get",
            Some(&Variant::from((PROXY_IFACE, "Accelerate"))),
            None,
            gio::DBusCallFlags::NONE,
            1000,
            gio::Cancellable::NONE,
        )
        .is_ok();
    if available {
        Some(Sensor::Proxy(connection))
    } else {
        None
    }
}

fn detect_sensor() -> Option<Sensor> {
    find_iio_accelerometer().or_else(sensor_proxy)
}

/// Starts automatic rotation when enabled. A sensor being present is enough to
/// enable it by default; `auto_rotate = false` in display.toml or
/// `BABYDRADE_AUTO_ROTATE=0` opts out.
pub(crate) fn spawn(event_loop: &mut EventLoop<'_, CalloopData>) {
    let enabled = match std::env::var("BABYDRADE_AUTO_ROTATE").as_deref() {
        Ok("0") | Ok("false") => false,
        Ok(_) => true,
        Err(_) => services::config::load_display_config()
            .auto_rotate
            .unwrap_or(true),
    };
    if !enabled {
        tracing::info!("auto-rotate disabled by configuration");
        return;
    }

    let Some(sensor) = detect_sensor() else {
        tracing::info!("auto-rotate: no accelerometer found (IIO or iio-sensor-proxy)");
        return;
    };

    let mut controller = AutoRotate::new(sensor);
    tracing::info!("auto-rotate enabled using {:?}", controller.sensor);
    if let Err(err) = event_loop.handle().insert_source(
        Timer::from_duration(POLL_INTERVAL),
        move |_, _, data: &mut CalloopData| {
            controller.poll(data);
            TimeoutAction::ToDuration(POLL_INTERVAL)
        },
    ) {
        tracing::warn!("cannot start auto-rotate timer: {err}");
    }
}

#[cfg(test)]
mod tests {
    use super::Orientation;
    use super::*;
    use smithay::utils::Transform;

    #[test]
    fn gravity_maps_to_quarters() {
        assert_eq!(Orientation::from_gravity(0.0, 1.0), Some(Orientation(0)));
        assert_eq!(Orientation::from_gravity(1.0, 0.0), Some(Orientation(3)));
        assert_eq!(Orientation::from_gravity(0.0, -1.0), Some(Orientation(2)));
        assert_eq!(Orientation::from_gravity(-1.0, 0.0), Some(Orientation(1)));
        assert_eq!(Orientation::from_gravity(0.0, 0.0), None);
    }

    #[test]
    fn rotation_cycles_through_all_quarters() {
        assert_eq!(rotate(Transform::Normal, 1), Transform::_90);
        assert_eq!(rotate(Transform::Normal, 4), Transform::Normal);
        assert_eq!(rotate(Transform::Flipped90, 1), Transform::Flipped180);
    }

    #[test]
    fn orientation_steps_wrap_around() {
        assert_eq!(Orientation(3).steps_from(Orientation(0)), 3);
        assert_eq!(Orientation(0).steps_from(Orientation(3)), 1);
    }
}
