//! Automatic panel rotation from an accelerometer (IIO) sensor.
//!
//! Convertible/tablet devices expose gravity data under
//! `/sys/bus/iio/devices/iio:deviceN/in_accel_{x,y}_raw`. A small timer polls
//! the sensor, debounces the reading and updates every Wayland output's
//! transform. The transform present at startup is used as the baseline, so the
//! feature also works when the panel is physically mounted sideways.

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Orientation(u8);

impl Orientation {
    /// Dominant gravity axis: Y up, X right, Y down, X left (clockwise steps).
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

struct AutoRotate {
    sensor: PathBuf,
    scale: f64,
    baseline: Option<Orientation>,
    base_transform: Option<Transform>,
    candidate: Option<Orientation>,
    stable: u8,
    applied: u8,
}

impl AutoRotate {
    fn new(sensor: PathBuf) -> Self {
        let scale = read_number(&sensor.join("in_accel_scale")).unwrap_or(1.0);
        Self {
            sensor,
            scale,
            baseline: None,
            base_transform: None,
            candidate: None,
            stable: 0,
            applied: 0,
        }
    }

    fn read_orientation(&self) -> Option<Orientation> {
        let x = read_number(&self.sensor.join("in_accel_x_raw"))? * self.scale;
        let y = read_number(&self.sensor.join("in_accel_y_raw"))? * self.scale;
        Orientation::from_gravity(x, y)
    }

    fn poll(&mut self, state: &mut CalloopData) {
        let Some(current) = self.read_orientation() else {
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
        // Touch input is transformed separately; keep it in sync with the panel.
        std::env::set_var("BABYDRADE_TOUCH_TRANSFORM", transform_name(transform));
        tracing::info!("auto-rotated panel to {transform:?}");
    }
}

fn baseline_transform(data: &CalloopData) -> Transform {
    data.state
        .space
        .outputs()
        .next()
        .map(|output| output.current_transform())
        .unwrap_or(Transform::Normal)
}

/// Rotates a transform clockwise by `steps` quarter turns.
fn rotate(transform: Transform, steps: u8) -> Transform {
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

fn transform_name(transform: Transform) -> &'static str {
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

fn read_number(path: &Path) -> Option<f64> {
    std::fs::read_to_string(path)
        .ok()?
        .trim()
        .parse::<f64>()
        .ok()
}

fn find_accelerometer() -> Option<PathBuf> {
    let root = Path::new("/sys/bus/iio/devices");
    let entries = std::fs::read_dir(root).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.join("in_accel_x_raw").is_file() && path.join("in_accel_y_raw").is_file() {
            return Some(path);
        }
    }
    None
}

/// Starts automatic rotation when enabled in `display.toml` or with
/// `BABYDRADE_AUTO_ROTATE=1`. Machines without an accelerometer are a no-op.
pub(crate) fn spawn(event_loop: &mut EventLoop<'_, CalloopData>) {
    let enabled = services::config::load_display_config().auto_rotate
        || std::env::var("BABYDRADE_AUTO_ROTATE").as_deref() == Ok("1");
    if !enabled {
        return;
    }

    let Some(sensor) = find_accelerometer() else {
        tracing::info!("auto-rotate enabled but no IIO accelerometer was found");
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
    use super::*;

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
}
