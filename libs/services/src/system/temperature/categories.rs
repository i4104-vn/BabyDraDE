//! Categorizes temperature sensors for desktop tooltips.

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
pub(super) fn category(label: &str) -> Option<&'static str> {
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
    // Wi-Fi/peripheral sensor readings are not useful in a desktop tooltip.
    None
}
