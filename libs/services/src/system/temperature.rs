//! System temperature readings from every source the kernel exposes.
//!
//! Thermal zones and hwmon sensors are both collected; on most machines they
//! describe overlapping hardware, so duplicate readings are dropped.

use std::fs;
use std::path::Path;

mod categories;
use categories::category;

/// Kernel subsystem that reported a temperature reading.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemperatureSource {
    Thermal,
    Hwmon,
}

/// A single temperature sensor reading.
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

/// Returns every temperature reading from thermal zones and hwmon.
///
/// # Returns
/// A vector of all deduplicated sensor readings.
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

/// Returns the most relevant temperature readings (CPU, GPU, SoC, etc.).
///
/// # Returns
/// A filtered vector of important sensor readings.
pub fn get_important_temperatures() -> Vec<TemperatureReading> {
    let order = ["CPU", "GPU", "SoC", "Storage", "System", "Battery"];
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
                .map(|(group, reading)| TemperatureReading {
                    label: group.to_string(),
                    celsius: reading.celsius,
                    source: reading.source,
                })
        })
        .take(6)
        .collect()
}

/// Reads the CPU temperature from the most representative sensor.
///
/// # Returns
/// The CPU temperature in degrees Celsius, or `None` if unavailable.
pub fn get_cpu_temperature() -> Option<f32> {
    let readings = get_important_temperatures();
    readings
        .iter()
        .find(|reading| reading.label == "CPU")
        .or_else(|| {
            readings
                .iter()
                .max_by(|a, b| a.celsius.total_cmp(&b.celsius))
        })
        .map(|reading| reading.celsius)
}
