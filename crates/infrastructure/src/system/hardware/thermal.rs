use std::fs;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ThermalZone {
    pub name: String,
    pub temp_celsius: f32,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ThermalInfo {
    pub max_temp_celsius: Option<f32>,
    pub zones: Vec<ThermalZone>,
}

#[derive(Debug, Default, Clone)]
pub struct ThermalReader;

impl ThermalReader {
    pub fn new() -> Self {
        Self
    }

    pub fn read(&self) -> ThermalInfo {
        let mut zones = Self::read_thermal_zones();

        // If no thermal_zones were found, try hwmon
        if zones.is_empty() {
            zones = Self::read_hwmon();
        }

        let max_temp_celsius = zones
            .iter()
            .map(|z| z.temp_celsius)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        ThermalInfo {
            max_temp_celsius,
            zones,
        }
    }

    fn read_thermal_zones() -> Vec<ThermalZone> {
        let mut zones = Vec::new();
        let entries = match fs::read_dir("/sys/class/thermal") {
            Ok(e) => e,
            Err(_) => return zones,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let name_str = match path.file_name() {
                Some(n) => n.to_string_lossy(),
                None => continue,
            };

            if !name_str.starts_with("thermal_zone") {
                continue;
            }

            let type_name = fs::read_to_string(path.join("type"))
                .unwrap_or_else(|_| name_str.to_string())
                .trim()
                .to_string();

            if let Ok(temp_str) = fs::read_to_string(path.join("temp")) {
                if let Ok(m_deg) = temp_str.trim().parse::<f32>() {
                    let temp_celsius = m_deg / 1000.0;
                    zones.push(ThermalZone {
                        name: type_name,
                        temp_celsius,
                    });
                }
            }
        }

        zones
    }

    fn read_hwmon() -> Vec<ThermalZone> {
        let mut zones = Vec::new();
        let entries = match fs::read_dir("/sys/class/hwmon") {
            Ok(e) => e,
            Err(_) => return zones,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let hwmon_name = fs::read_to_string(path.join("name"))
                .unwrap_or_else(|_| "hwmon".to_string())
                .trim()
                .to_string();

            // Look for temp*_input files
            if let Ok(hw_entries) = fs::read_dir(&path) {
                for hw_entry in hw_entries.flatten() {
                    let file_name = hw_entry.file_name().to_string_lossy().to_string();
                    if file_name.starts_with("temp") && file_name.ends_with("_input") {
                        let label_file = file_name.replace("_input", "_label");
                        let label = fs::read_to_string(path.join(&label_file))
                            .map(|s| s.trim().to_string())
                            .unwrap_or_else(|_| format!("{}_{}", hwmon_name, file_name));

                        if let Ok(temp_str) = fs::read_to_string(hw_entry.path()) {
                            if let Ok(m_deg) = temp_str.trim().parse::<f32>() {
                                zones.push(ThermalZone {
                                    name: label,
                                    temp_celsius: m_deg / 1000.0,
                                });
                            }
                        }
                    }
                }
            }
        }

        zones
    }
}
