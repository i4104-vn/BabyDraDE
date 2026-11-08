use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatteryStatus {
    Charging,
    Discharging,
    Full,
    NotCharging,
    Unknown,
}

impl Default for BatteryStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

impl std::fmt::Display for BatteryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Charging => write!(f, "Charging"),
            Self::Discharging => write!(f, "Discharging"),
            Self::Full => write!(f, "Full"),
            Self::NotCharging => write!(f, "Not Charging"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BatteryInfo {
    pub name: String,
    pub capacity: u8,
    pub status: BatteryStatus,
    pub is_charging: bool,
    pub power_watts: Option<f32>,
    pub energy_now_wh: Option<f32>,
    pub energy_full_wh: Option<f32>,
}

#[derive(Debug, Default, Clone)]
pub struct BatteryReader {
    device_path: Option<PathBuf>,
}

impl BatteryReader {
    pub fn new() -> Self {
        let device_path = Self::find_battery_device();
        Self { device_path }
    }

    pub fn read(&mut self) -> Option<BatteryInfo> {
        // Re-check path if not cached or verify it still exists
        let path = match self.device_path {
            Some(ref p) if p.exists() => p.clone(),
            _ => {
                let found = Self::find_battery_device()?;
                self.device_path = Some(found.clone());
                found
            }
        };

        Self::read_battery(&path)
    }

    fn find_battery_device() -> Option<PathBuf> {
        let entries = fs::read_dir("/sys/class/power_supply").ok()?;

        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = path.file_name()?.to_string_lossy();

            // Typically BAT0, BAT1, or device with type == Battery
            if file_name.starts_with("BAT") {
                return Some(path);
            }

            // Also check type file if present
            if let Ok(dev_type) = fs::read_to_string(path.join("type")) {
                if dev_type.trim() == "Battery" {
                    return Some(path);
                }
            }
        }

        None
    }

    fn read_battery(path: &Path) -> Option<BatteryInfo> {
        let name = path.file_name()?.to_string_lossy().to_string();

        let capacity_str = fs::read_to_string(path.join("capacity")).ok()?;
        let capacity: u8 = capacity_str.trim().parse().ok()?;

        let status_str = fs::read_to_string(path.join("status"))
            .unwrap_or_default()
            .trim()
            .to_lowercase();

        let status = match status_str.as_str() {
            "charging" => BatteryStatus::Charging,
            "discharging" => BatteryStatus::Discharging,
            "full" => BatteryStatus::Full,
            "not charging" => BatteryStatus::NotCharging,
            _ => BatteryStatus::Unknown,
        };

        let is_charging = matches!(status, BatteryStatus::Charging);

        let power_watts = Self::read_power_watts(path);
        let energy_now_wh = Self::read_energy_wh(path, "energy_now", "charge_now");
        let energy_full_wh = Self::read_energy_wh(path, "energy_full", "charge_full");

        Some(BatteryInfo {
            name,
            capacity: capacity.min(100),
            status,
            is_charging,
            power_watts,
            energy_now_wh,
            energy_full_wh,
        })
    }

    fn read_power_watts(path: &Path) -> Option<f32> {
        // Try power_now (microwatts)
        if let Ok(content) = fs::read_to_string(path.join("power_now")) {
            if let Ok(u_watts) = content.trim().parse::<f64>() {
                return Some((u_watts / 1_000_000.0) as f32);
            }
        }

        // Try current_now (microamps) * voltage_now (microvolts)
        let current_u = fs::read_to_string(path.join("current_now"))
            .ok()?
            .trim()
            .parse::<f64>()
            .ok()?;
        let voltage_u = fs::read_to_string(path.join("voltage_now"))
            .ok()?
            .trim()
            .parse::<f64>()
            .ok()?;

        let watts = (current_u * voltage_u) / 1_000_000_000_000.0;
        Some(watts as f32)
    }

    fn read_energy_wh(path: &Path, energy_file: &str, charge_file: &str) -> Option<f32> {
        // Try direct energy in micro-watt-hours
        if let Ok(content) = fs::read_to_string(path.join(energy_file)) {
            if let Ok(u_wh) = content.trim().parse::<f64>() {
                return Some((u_wh / 1_000_000.0) as f32);
            }
        }

        // Try charge in micro-amp-hours * voltage_min_design or voltage_now
        let charge_u = fs::read_to_string(path.join(charge_file))
            .ok()?
            .trim()
            .parse::<f64>()
            .ok()?;
        let voltage_u = fs::read_to_string(path.join("voltage_min_design"))
            .or_else(|_| fs::read_to_string(path.join("voltage_now")))
            .ok()?
            .trim()
            .parse::<f64>()
            .ok()?;

        let wh = (charge_u * voltage_u) / 1_000_000_000_000.0;
        Some(wh as f32)
    }
}
