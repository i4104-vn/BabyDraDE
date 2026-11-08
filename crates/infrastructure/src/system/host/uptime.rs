use std::fs;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UptimeInfo {
    pub uptime_seconds: u64,
    pub formatted: String,
    pub hostname: String,
}

#[derive(Debug, Default, Clone)]
pub struct UptimeReader;

impl UptimeReader {
    pub fn new() -> Self {
        Self
    }

    pub fn read(&self) -> UptimeInfo {
        let uptime_seconds = Self::read_uptime_seconds().unwrap_or(0);
        let formatted = Self::format_uptime(uptime_seconds);
        let hostname = Self::read_hostname().unwrap_or_else(|| "localhost".to_string());

        UptimeInfo {
            uptime_seconds,
            formatted,
            hostname,
        }
    }

    fn read_uptime_seconds() -> Option<u64> {
        let content = fs::read_to_string("/proc/uptime").ok()?;
        let first_val = content.split_whitespace().next()?;
        let seconds_f: f64 = first_val.parse().ok()?;
        Some(seconds_f as u64)
    }

    fn read_hostname() -> Option<String> {
        if let Ok(name) = fs::read_to_string("/proc/sys/kernel/hostname") {
            let trimmed = name.trim().to_string();
            if !trimmed.is_empty() {
                return Some(trimmed);
            }
        }

        if let Ok(name) = fs::read_to_string("/etc/hostname") {
            let trimmed = name.trim().to_string();
            if !trimmed.is_empty() {
                return Some(trimmed);
            }
        }

        None
    }

    pub fn format_uptime(total_seconds: u64) -> String {
        let days = total_seconds / 86400;
        let hours = (total_seconds % 86400) / 3600;
        let minutes = (total_seconds % 3600) / 60;

        match (days, hours) {
            (d, h) if d > 0 => format!("{d}d {h}h {minutes}m"),
            (0, h) if h > 0 => format!("{h}h {minutes}m"),
            _ => format!("{minutes}m"),
        }
    }
}
