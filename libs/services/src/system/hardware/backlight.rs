use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Brightness information for a backlight device.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BacklightInfo {
    pub device_name: String,
    pub brightness: u64,
    pub max_brightness: u64,
    pub percent: u8,
}

/// Discovers and reads the system backlight device.
/// Discovers and reads the system backlight device.
#[derive(Debug, Default, Clone)]
pub struct BacklightReader {
    device_path: Option<PathBuf>,
}

/// Converts a brightness percentage into a raw sysfs value.
///
/// Returns `None` when the device reports no usable maximum.
fn brightness_raw(target_percent: u8, max_brightness: u64) -> Option<u64> {
    if max_brightness == 0 {
        return None;
    }
    let clamped = target_percent.min(100) as f64;
    Some((clamped / 100.0 * max_brightness as f64).round() as u64)
}

/// Reads and writes brightness through sysfs.
impl BacklightReader {
    /// Creates a reader, probing `/sys/class/backlight` for a device.
    ///
    /// # Returns
    /// A reader with the detected device cached, if any.
    /// Creates a reader, probing `/sys/class/backlight` for a device.
    ///
    /// # Returns
    /// A reader with the detected device cached, if any.
    pub fn new() -> Self {
        let device_path = Self::find_backlight_device();
        Self { device_path }
    }

    /// Reads current brightness, re-probing if the cached device disappeared.
    ///
    /// # Returns
    /// The brightness snapshot, or `None` when no readable device exists.
    /// Reads current brightness, re-probing if the cached device disappeared.
    ///
    /// # Returns
    /// The brightness snapshot, or `None` when no readable device exists.
    pub fn read(&mut self) -> Option<BacklightInfo> {
        let path = match self.device_path {
            Some(ref p) if p.exists() => p.clone(),
            _ => {
                let found = Self::find_backlight_device()?;
                self.device_path = Some(found.clone());
                found
            }
        };

        Self::read_device(&path)
    }

    /// Sets brightness to a percentage of the device maximum.
    ///
    /// # Arguments
    /// * `target_percent` - Desired percentage, clamped to `0..=100`.
    ///
    /// # Errors
    /// Returns an error when no device exists, sysfs cannot be read/written,
    /// or `max_brightness` is zero.
    /// Sets brightness to a percentage of the device maximum.
    ///
    /// # Arguments
    /// * `target_percent` - Desired percentage, clamped to `0..=100`.
    ///
    /// # Errors
    /// Returns an error when no device exists, sysfs cannot be read/written,
    /// or `max_brightness` is zero.
    pub fn set_brightness_percent(&mut self, target_percent: u8) -> io::Result<()> {
        let sysfs_result = self.write_sysfs_brightness(target_percent);
        if sysfs_result.is_ok() {
            return Ok(());
        }

        // Direct sysfs writes often require root or a udev rule. Fall back to
        // the userspace tools that handle permissions themselves.
        let percent = format!("{}%", target_percent.min(100));
        let via_brightnessctl = Command::new("brightnessctl")
            .arg("set")
            .arg(&percent)
            .output()
            .is_ok_and(|output| output.status.success());
        if via_brightnessctl {
            return Ok(());
        }
        let via_xbacklight = Command::new("xbacklight")
            .args(["-set", &percent])
            .output()
            .is_ok_and(|output| output.status.success());
        if via_xbacklight {
            return Ok(());
        }

        tracing::warn!(
            "could not set brightness: sysfs and userspace tools all failed ({})",
            sysfs_result.as_ref().err().map_or_else(
                || "unknown error".to_string(),
                |err| err.to_string()
            )
        );
        sysfs_result
    }

    fn write_sysfs_brightness(&mut self, target_percent: u8) -> io::Result<()> {
        let path = match self.device_path {
            Some(ref p) if p.exists() => p.clone(),
            _ => {
                let found = Self::find_backlight_device().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::NotFound, "No backlight device found")
                })?;
                self.device_path = Some(found.clone());
                found
            }
        };

        let max_str = fs::read_to_string(path.join("max_brightness"))?;
        let max_brightness: u64 = max_str
            .trim()
            .parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let target_raw = brightness_raw(target_percent, max_brightness)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "max_brightness is 0"))?;

        fs::write(path.join("brightness"), target_raw.to_string())
    }

    fn find_backlight_device() -> Option<PathBuf> {
        let entries = fs::read_dir("/sys/class/backlight").ok()?;
        let mut first: Option<PathBuf> = None;

        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name()?.to_string_lossy().to_lowercase();

            // Prefer vendor drivers like intel_backlight or amdgpu_bl over generic acpi
            if name.contains("intel") || name.contains("amdgpu") || name.contains("nvidia") {
                return Some(path);
            }

            if first.is_none() {
                first = Some(path);
            }
        }

        first
    }

    fn read_device(path: &Path) -> Option<BacklightInfo> {
        let device_name = path.file_name()?.to_string_lossy().to_string();

        let brightness_str = fs::read_to_string(path.join("brightness")).ok()?;
        let brightness: u64 = brightness_str.trim().parse().ok()?;

        let max_str = fs::read_to_string(path.join("max_brightness")).ok()?;
        let max_brightness: u64 = max_str.trim().parse().ok()?;

        if max_brightness == 0 {
            return None;
        }

        let percent = ((brightness as f64 / max_brightness as f64) * 100.0).round() as u8;

        Some(BacklightInfo {
            device_name,
            brightness,
            max_brightness,
            percent: percent.min(100),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_percentage_to_raw_value() {
        assert_eq!(brightness_raw(50, 200), Some(100));
        assert_eq!(brightness_raw(100, 250), Some(250));
        assert_eq!(brightness_raw(1, 1000), Some(10));
    }

    #[test]
    fn clamps_to_device_maximum() {
        assert_eq!(brightness_raw(200, 200), Some(200));
    }

    #[test]
    fn rejects_zero_maximum() {
        assert_eq!(brightness_raw(50, 0), None);
    }
}
