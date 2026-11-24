use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Brightness information for a backlight device.
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

/// Reads and writes brightness through sysfs.
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

        if max_brightness == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "max_brightness is 0",
            ));
        }

        let clamped = target_percent.min(100);
        let target_raw = ((clamped as f64 / 100.0) * max_brightness as f64).round() as u64;

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
