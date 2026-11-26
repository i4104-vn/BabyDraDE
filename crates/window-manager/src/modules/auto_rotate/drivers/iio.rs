//! Raw IIO sysfs accelerometer driver.

use std::path::{Path, PathBuf};

use crate::modules::auto_rotate::{driver::RotationDriver, models::Gravity};

const IIO_ROOTS: [&str; 2] = ["/sys/bus/iio/devices", "/sys/class/iio"];
const SCALE_FILES: [&str; 3] = ["in_accel_scale", "in_accel_x_scale", "in_accel_y_scale"];

/// Reads a numeric sysfs attribute.
fn read_number(path: &Path) -> Option<f64> {
    std::fs::read_to_string(path)
        .ok()?
        .trim()
        .parse::<f64>()
        .ok()
}

/// Resolves the scale used by an IIO device; fall back to raw 1g units.
fn read_scale(path: &Path) -> f64 {
    SCALE_FILES
        .iter()
        .find_map(|name| read_number(&path.join(name)))
        .unwrap_or(1.0)
}

pub(crate) struct IioDriver {
    path: PathBuf,
    scale: f64,
}

impl IioDriver {
    /// Finds the first sysfs device exposing a two-axis accelerometer.
    ///
    /// # Returns
    /// A boxed driver, or `None` when no usable IIO device is present.
    pub(crate) fn probe() -> Option<Box<dyn RotationDriver>> {
        for root in IIO_ROOTS {
            let Ok(entries) = std::fs::read_dir(root) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.join("in_accel_x_raw").is_file() && path.join("in_accel_y_raw").is_file() {
                    let scale = read_scale(&path);
                    return Some(Box::new(Self { path, scale }));
                }
            }
        }
        None
    }
}

impl RotationDriver for IioDriver {
    fn name(&self) -> &str {
        "iio"
    }

    fn read(&mut self) -> Option<Gravity> {
        let x = read_number(&self.path.join("in_accel_x_raw"))? * self.scale;
        let y = read_number(&self.path.join("in_accel_y_raw"))? * self.scale;
        Some(Gravity { x, y })
    }
}

#[cfg(test)]
mod tests {
    use super::{read_number, read_scale};
    use std::fs;
    use std::path::PathBuf;

    fn temp_dir(tag: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("babydra-iio-{tag}-{}", std::process::id()));
        fs::create_dir_all(&path).expect("create temp dir");
        path
    }

    #[test]
    fn read_number_rejects_non_numeric_sysfs_values() {
        let dir = temp_dir("invalid");
        let file = dir.join("value");
        fs::write(&file, "not-a-number").expect("write temp file");
        assert_eq!(read_number(&file), None);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn read_scale_accepts_axis_specific_attribute() {
        let dir = temp_dir("scale");
        fs::write(dir.join("in_accel_x_scale"), "0.004\n").expect("write temp file");
        assert_eq!(read_scale(&dir), 0.004);
        let _ = fs::remove_dir_all(dir);
    }
}
