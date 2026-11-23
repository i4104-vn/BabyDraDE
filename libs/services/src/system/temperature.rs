//! System temperature readings from every source the kernel exposes.
//!
//! Thermal zones and hwmon sensors are both collected; on most machines they
//! describe overlapping hardware, so duplicate readings are dropped.

use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemperatureSource {
    Thermal,
    Hwmon,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TemperatureReading {
    pub label: String,
    pub celsius: f32,
    pub source: TemperatureSource,
}

fn read_celsius(path: &Path, millidegrees: bool) -> Option<f32> {
    let value: f32 = fs::read_to_string(path).ok()?.trim().parse().ok()?;
    Some(if millidegrees { value / 1000.0 } else { value })
}

fn thermal_zones() -> Vec<TemperatureReading> {
    let mut readings = Vec::new();
    let Ok(entries) = fs::read_dir("/sys/class/thermal") else {
        return readings;
    };

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("thermal_zone") {
            continue;
        }
        let path = entry.path();
        let label = fs::read_to_string(path.join("type"))
            .unwrap_or_else(|_| name.clone())
            .trim()
            .to_string();
        if let Some(celsius) = read_celsius(&path.join("temp"), true) {
            readings.push(TemperatureReading {
                label,
                celsius,
                source: TemperatureSource::Thermal,
            });
        }
    }
    readings
}

fn hwmon_sensors() -> Vec<TemperatureReading> {
    let mut readings = Vec::new();
    let Ok(entries) = fs::read_dir("/sys/class/hwmon") else {
        return readings;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let chip = fs::read_to_string(path.join("name"))
            .unwrap_or_else(|_| "hwmon".to_string())
            .trim()
            .to_string();
        let Ok(sensors) = fs::read_dir(&path) else {
            continue;
        };

        for sensor in sensors.flatten() {
            let file_name = sensor.file_name().to_string_lossy().to_string();
            if !file_name.starts_with("temp") || !file_name.ends_with("_input") {
                continue;
            }
            let celsius = match read_celsius(&sensor.path(), true) {
                Some(value) => value,
                None => continue,
            };
            let label = fs::read_to_string(path.join(file_name.replace("_input", "_label")))
                .map(|label| label.trim().to_string())
                .unwrap_or_else(|_| format!("{chip} {}", file_name.trim_end_matches("_input")));
            readings.push(TemperatureReading {
                label,
                celsius,
                source: TemperatureSource::Hwmon,
            });
        }
    }
    readings
}

/// Sensors that are not meaningful to show in a desktop tooltip (peripheral
/// batteries, fan controllers and unnamed hwmon placeholders).
fn is_ignored(label: &str) -> bool {
    let label = label.to_lowercase();
    label.contains("hidpp")
        || label.contains("fan")
        || label.contains("rpm")
        || label.contains("unknown")
        || (label.starts_with("temp") && label.ends_with("input"))
}

/// Groups a useful reading into a single category so a CPU with 16 cores does
/// not fill the tooltip with duplicates.
fn category(label: &str) -> Option<&'static str> {
    let label = label.to_lowercase();
    if is_ignored(&label) {
        return None;
    }
    for needle in [
        "package",
        "tctl",
        "cpu_thermal",
        "k10temp",
        "zenpower",
        "cpu",
        "core",
    ] {
        if label.contains(needle) {
            return Some("CPU");
        }
    }
    for needle in ["gpu", "nvidia", "amdgpu", "radeon", "xe", "intel_gt"] {
        if label.contains(needle) {
            return Some("GPU");
        }
    }
    for needle in ["soc", "apu"] {
        if label.contains(needle) {
            return Some("SoC");
        }
    }
    for needle in ["nvme", "ssd", "hdd", "drive"] {
        if label.contains(needle) {
            return Some("Storage");
        }
    }
    for needle in ["pch", "chipset", "acpitz", "dell_smm", "platform"] {
        if label.contains(needle) {
            return Some("System");
        }
    }
    if label.contains("battery") {
        return Some("Battery");
    }
    for needle in ["wifi", "iwlwifi", "ath", "wlan"] {
        if label.contains(needle) {
            return Some("Wireless");
        }
    }
    None
}

/// Returns every temperature reading thermal zones and hwmon chips provide,
/// sorted by label. Duplicate labels from the two sources are kept only once.
pub fn get_all_temperatures() -> Vec<TemperatureReading> {
    let mut readings = thermal_zones();
    readings.extend(hwmon_sensors());
    readings.sort_by(|a, b| {
        a.label
            .cmp(&b.label)
            .then_with(|| a.celsius.total_cmp(&b.celsius))
    });
    readings.dedup_by(|a, b| a.label == b.label && a.celsius == b.celsius);
    readings
}

/// A short, curated list for tooltips: one reading per category (CPU, GPU,
/// SoC, Storage, System, Battery, Wireless), capped at six rows.
pub fn get_important_temperatures() -> Vec<TemperatureReading> {
    let order = [
        "CPU", "GPU", "SoC", "Storage", "System", "Battery", "Wireless",
    ];
    let mut selected: Vec<(&str, TemperatureReading)> = Vec::new();

    for reading in get_all_temperatures() {
        let Some(group) = category(&reading.label) else {
            continue;
        };
        match selected.iter_mut().find(|(current, _)| *current == group) {
            Some((_, existing)) if reading.celsius > existing.celsius => *existing = reading,
            Some(_) => {}
            None => selected.push((group, reading)),
        }
    }

    order
        .iter()
        .filter_map(|group| {
            selected
                .iter()
                .find(|(current, _)| current == group)
                .map(|(_, reading)| reading.clone())
        })
        .take(6)
        .collect()
}

/// Picks the reading that represents the CPU package. Falls back to the
/// hottest sensor on machines whose firmware does not label it.
pub fn get_cpu_temperature() -> Option<f32> {
    let readings = get_important_temperatures();
    readings
        .iter()
        .find(|reading| category(&reading.label) == Some("CPU"))
        .or_else(|| {
            readings
                .iter()
                .max_by(|a, b| a.celsius.total_cmp(&b.celsius))
        })
        .map(|reading| reading.celsius)
}
