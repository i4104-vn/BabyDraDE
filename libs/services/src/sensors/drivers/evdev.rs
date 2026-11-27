//! Linux `evdev` accelerometer driver (`/dev/input/event*`).
//!
//! Covers laptops and tablets whose accelerometer is exposed through the
//! input subsystem (`INPUT_PROP_ACCELEROMETER`) rather than IIO sysfs, such as
//! `lis3lv02d`, `hp_accel`, `hdaps`, `applesmc` and `toshiba_acpi`, when
//! `iio-sensor-proxy` is not running.

use std::fs::File;
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};

use crate::sensors::{Gravity, RotationDriver};

/// `_IOC_READ | _IOC_WRITE`: the direction value of an ioctl request code.
const IOC_READ_WRITE: u64 = 2;
/// Bit position of the direction field in an ioctl request code.
const IOC_DIR_SHIFT: u32 = 30;
/// Bit position of the buffer-size field in an ioctl request code.
const IOC_SIZE_SHIFT: u32 = 16;
/// Bit position of the type field in an ioctl request code.
const IOC_TYPE_SHIFT: u32 = 8;
/// Base request number of the `EVIOCGABS(abs)` ioctl family.
const EVIOCGABS_BASE: u16 = 0x40;

const INPUT_CLASS_DIR: &str = "/sys/class/input";
/// `INPUT_PROP_ACCELEROMETER` bit in a sysfs `properties` bitmask.
const INPUT_PROP_ACCELEROMETER: u64 = 1 << 6;
const ABS_X: u16 = 0x00;
const ABS_Y: u16 = 0x01;
const ABS_Z: u16 = 0x02;

/// Checks whether a sysfs `properties` hex bitmask includes `INPUT_PROP_ACCELEROMETER`.
fn has_accelerometer_prop(raw: &str) -> bool {
    let Some(last_word) = raw.split_whitespace().last() else {
        return false;
    };
    u64::from_str_radix(last_word, 16)
        .map(|mask| (mask & INPUT_PROP_ACCELEROMETER) != 0)
        .unwrap_or(false)
}

/// Computes the Linux `EVIOCGABS(abs)` ioctl request code.
fn eviocgabs(abs: u16) -> libc::Ioctl {
    let size = std::mem::size_of::<libc::input_absinfo>() as libc::c_ulong;
    let req = (IOC_READ_WRITE << IOC_DIR_SHIFT)
        | (u64::from(size as u32) << IOC_SIZE_SHIFT)
        | (u64::from(b'E') << IOC_TYPE_SHIFT)
        | u64::from(EVIOCGABS_BASE + abs);
    req as libc::Ioctl
}

/// Reads an `ABS_*` axis value from an open evdev file descriptor.
fn read_abs(file: &File, abs: u16) -> Option<f64> {
    let mut info = libc::input_absinfo {
        value: 0,
        minimum: 0,
        maximum: 0,
        fuzz: 0,
        flat: 0,
        resolution: 0,
    };
    // SAFETY: `file` is a valid open evdev fd and `info` is an out-pointer for `EVIOCGABS`.
    let rc = unsafe { libc::ioctl(file.as_raw_fd(), eviocgabs(abs), &mut info) };
    if rc < 0 {
        None
    } else {
        Some(f64::from(info.value))
    }
}

/// Finds the `/dev/input/eventN` path for the first accelerometer input device.
fn find_accel_event_node() -> Option<PathBuf> {
    let entries = std::fs::read_dir(INPUT_CLASS_DIR).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if !name_str.starts_with("event") {
            continue;
        }
        let props_path = entry.path().join("device/properties");
        let Ok(props) = std::fs::read_to_string(props_path) else {
            continue;
        };
        if has_accelerometer_prop(&props) {
            let dev_path = Path::new("/dev/input").join(name_str.as_ref());
            if dev_path.exists() {
                return Some(dev_path);
            }
        }
    }
    None
}

#[derive(Debug)]
pub(crate) struct EvdevDriver {
    file: File,
}

impl EvdevDriver {
    /// Probes `/sys/class/input/event*` for a device with `INPUT_PROP_ACCELEROMETER`.
    pub(crate) fn probe() -> Option<Box<dyn RotationDriver>> {
        let path = find_accel_event_node()?;
        let file = File::open(path).ok()?;
        // Verify we can read at least X and Y axes before claiming the driver.
        read_abs(&file, ABS_X)?;
        read_abs(&file, ABS_Y)?;
        Some(Box::new(Self { file }))
    }
}

impl RotationDriver for EvdevDriver {
    fn name(&self) -> &str {
        "evdev"
    }

    fn read(&mut self) -> Option<Gravity> {
        let x = read_abs(&self.file, ABS_X)?;
        let y = read_abs(&self.file, ABS_Y)?;
        let z = read_abs(&self.file, ABS_Z).unwrap_or(0.0);
        let mag = (x * x + y * y + z * z).sqrt();
        if mag > 0.0 {
            Some(Gravity {
                x: x / mag,
                y: y / mag,
            })
        } else {
            Some(Gravity { x: 0.0, y: 0.0 })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::has_accelerometer_prop;

    #[test]
    fn detects_accelerometer_input_property() {
        assert!(has_accelerometer_prop("40\n"));
        assert!(has_accelerometer_prop("0 41\n"));
        assert!(!has_accelerometer_prop("0\n"));
        assert!(!has_accelerometer_prop("1\n"));
    }
}
