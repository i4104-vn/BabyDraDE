//! Data source for the battery indicator (read by `crate::center::poller`).

use std::path::Path;

use services::system::hardware::BatteryReader;

#[derive(Debug, Clone, Copy)]
pub struct BatteryInfo {
    pub percentage: u32,
    pub is_charging: bool,
    pub is_ac_only: bool,
}

/// Reads the primary battery through BabyDraDE's hardware service.
pub fn read() -> Option<BatteryInfo> {
    let mut reader = BatteryReader::new();
    reader.read().map(|info| BatteryInfo {
        percentage: u32::from(info.capacity),
        is_charging: info.is_charging,
        is_ac_only: false,
    })
}

/// Checks whether any AC power supply / mains adapter is online.
pub fn is_ac_plugged() -> bool {
    let power_dir = Path::new("/sys/class/power_supply");
    if !power_dir.exists() {
        return false;
    }
    let Ok(entries) = std::fs::read_dir(power_dir) else {
        return false;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if let Ok(kind) = std::fs::read_to_string(path.join("type")) {
            let kind = kind.trim();
            if kind == "Mains" || kind == "AC" {
                if let Ok(online) = std::fs::read_to_string(path.join("online")) {
                    if online.trim() == "1" {
                        return true;
                    }
                }
            }
        }
    }
    false
}
