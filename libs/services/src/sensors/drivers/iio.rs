//! Raw Linux IIO sysfs accelerometer driver.
//!
//! Supports `_raw` + `_scale` (per-axis or shared) as well as processed `_input`
//! attributes, applies `in_accel_mount_matrix` / `mount_matrix` when exposed by
//! the kernel, prefers display/lid sensors over base sensors on dual-accel
//! convertibles, and normalizes 3-axis readings so flat detection is
//! unit-independent.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::sensors::{Gravity, RotationDriver};

/// Sysfs roots that may expose IIO accelerometer devices.
const IIO_ROOTS: [&str; 2] = ["/sys/bus/iio/devices", "/sys/class/iio"];
/// Standard gravity in m/s², used to normalize scaled `_raw` samples.
const EARTH_GRAVITY: f64 = 9.80665;
/// Row-major identity matrix used when the device exposes no mount matrix.
const IDENTITY_MATRIX: [f64; 9] = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];

/// Reads a numeric sysfs attribute.
fn read_number(path: &Path) -> Option<f64> {
    std::fs::read_to_string(path)
        .ok()?
        .trim()
        .parse::<f64>()
        .ok()
}

/// Reads a trimmed string sysfs attribute.
fn read_text(path: &Path) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_ascii_lowercase())
    }
}

/// Resolves the scale for a single axis, falling back to the shared scale or 1.0.
fn read_axis_scale(path: &Path, axis: &str) -> (f64, bool) {
    let specific = format!("in_accel_{axis}_scale");
    if let Some(scale) = read_number(&path.join(specific)) {
        return (scale, true);
    }
    if let Some(scale) = read_number(&path.join("in_accel_scale")) {
        return (scale, true);
    }
    (1.0, false)
}

/// Parses a 3x3 IIO mount matrix (`a, b, c; d, e, f; g, h, i`).
fn parse_mount_matrix(raw: &str) -> Option<[f64; 9]> {
    let values: Vec<f64> = raw
        .split([';', ','])
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(str::parse::<f64>)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    values.try_into().ok()
}

/// Reads `in_accel_mount_matrix` or `mount_matrix` if present, else identity.
fn read_mount_matrix(path: &Path) -> [f64; 9] {
    ["in_accel_mount_matrix", "mount_matrix"]
        .iter()
        .find_map(|name| std::fs::read_to_string(path.join(name)).ok())
        .and_then(|raw| parse_mount_matrix(&raw))
        .unwrap_or(IDENTITY_MATRIX)
}

/// Applies a row-major 3x3 mount matrix to a 3-axis acceleration vector.
fn apply_matrix(m: &[f64; 9], x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    (
        m[0] * x + m[1] * y + m[2] * z,
        m[3] * x + m[4] * y + m[5] * z,
        m[6] * x + m[7] * y + m[8] * z,
    )
}

#[derive(Clone, Copy)]
enum ChannelMode {
    Raw,
    Input,
}

impl ChannelMode {
    fn attr(self, axis: &str) -> String {
        match self {
            Self::Raw => format!("in_accel_{axis}_raw"),
            Self::Input => format!("in_accel_{axis}_input"),
        }
    }
}

/// Scores an IIO device so display/lid sensors beat base sensors.
fn device_priority(path: &Path) -> u8 {
    let loc = read_text(&path.join("location")).or_else(|| read_text(&path.join("label")));
    match loc.as_deref() {
        Some("display") | Some("lid") | Some("screen") => 2,
        Some("base") | Some("keyboard") => 0,
        _ => 1,
    }
}

/// Inspects a candidate IIO directory and builds a driver if usable.
fn inspect_device(path: &Path) -> Option<(u8, IioDriver)> {
    let mode = if path.join("in_accel_x_raw").is_file() && path.join("in_accel_y_raw").is_file() {
        ChannelMode::Raw
    } else if path.join("in_accel_x_input").is_file() && path.join("in_accel_y_input").is_file() {
        ChannelMode::Input
    } else {
        return None;
    };

    let (scale_x, has_scale_x) = match mode {
        ChannelMode::Raw => read_axis_scale(path, "x"),
        ChannelMode::Input => (1.0, true),
    };
    let (scale_y, has_scale_y) = match mode {
        ChannelMode::Raw => read_axis_scale(path, "y"),
        ChannelMode::Input => (1.0, true),
    };
    let (scale_z, _) = match mode {
        ChannelMode::Raw => read_axis_scale(path, "z"),
        ChannelMode::Input => (1.0, true),
    };
    let has_z = path.join(mode.attr("z")).is_file();
    let matrix = read_mount_matrix(path);

    Some((
        device_priority(path),
        IioDriver {
            path: path.to_path_buf(),
            mode,
            scale_x,
            scale_y,
            scale_z,
            has_scale: has_scale_x && has_scale_y,
            has_z,
            matrix,
        },
    ))
}

pub(crate) struct IioDriver {
    path: PathBuf,
    mode: ChannelMode,
    scale_x: f64,
    scale_y: f64,
    scale_z: f64,
    has_scale: bool,
    has_z: bool,
    matrix: [f64; 9],
}

impl IioDriver {
    /// Finds the best sysfs device exposing an accelerometer.
    ///
    /// # Returns
    /// A boxed driver, or `None` when no usable IIO device is present.
    pub(crate) fn probe() -> Option<Box<dyn RotationDriver>> {
        let mut seen = HashSet::new();
        let mut best: Option<(u8, IioDriver)> = None;

        for root in IIO_ROOTS {
            let Ok(entries) = std::fs::read_dir(root) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                let canonical = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
                if !seen.insert(canonical) {
                    continue;
                }
                if let Some((prio, driver)) = inspect_device(&path) {
                    if best.as_ref().is_none_or(|(best_prio, _)| prio > *best_prio) {
                        best = Some((prio, driver));
                    }
                }
            }
        }
        best.map(|(_, driver)| Box::new(driver) as Box<dyn RotationDriver>)
    }
}

impl RotationDriver for IioDriver {
    fn name(&self) -> &str {
        "iio"
    }

    fn read(&mut self) -> Option<Gravity> {
        let raw_x = read_number(&self.path.join(self.mode.attr("x")))? * self.scale_x;
        let raw_y = read_number(&self.path.join(self.mode.attr("y")))? * self.scale_y;
        let raw_z = if self.has_z {
            read_number(&self.path.join(self.mode.attr("z"))).unwrap_or(0.0) * self.scale_z
        } else {
            0.0
        };

        let (x, y, z) = apply_matrix(&self.matrix, raw_x, raw_y, raw_z);
        if self.has_z {
            let mag = (x * x + y * y + z * z).sqrt();
            if mag > 0.0 {
                return Some(Gravity {
                    x: x / mag,
                    y: y / mag,
                });
            }
        }
        let divisor = if self.has_scale { EARTH_GRAVITY } else { 1.0 };
        Some(Gravity {
            x: x / divisor,
            y: y / divisor,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{apply_matrix, parse_mount_matrix, read_axis_scale, read_number};
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
        assert_eq!(read_axis_scale(&dir, "x"), (0.004, true));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn parses_and_applies_mount_matrix() {
        let matrix = parse_mount_matrix("0, 1, 0; -1, 0, 0; 0, 0, 1").expect("valid matrix");
        assert_eq!(apply_matrix(&matrix, 1.0, 2.0, 3.0), (2.0, -1.0, 3.0));
    }
}
