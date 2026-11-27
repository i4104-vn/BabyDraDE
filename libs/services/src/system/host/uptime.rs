use std::fs;

/// Seconds in a civil day (24 × 60 × 60); changes the day component of labels.
const SECONDS_PER_DAY: u64 = 86_400;
/// Seconds in an hour; changes the hour component of uptime labels.
const SECONDS_PER_HOUR: u64 = 3_600;
/// Seconds in a minute; changes the minute component of uptime labels.
const SECONDS_PER_MINUTE: u64 = 60;

/// System uptime and hostname information.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UptimeInfo {
    pub uptime_seconds: u64,
    pub formatted: String,
    pub hostname: String,
}

/// Reads system uptime and hostname from procfs.
#[derive(Debug, Default, Clone)]
pub struct UptimeReader;

/// Provides uptime and hostname queries.
impl UptimeReader {
    /// Creates an uptime reader.
    ///
    /// # Returns
    /// A stateless reader instance.
    pub fn new() -> Self {
        Self
    }

    /// Reads uptime and hostname, falling back to defaults on failure.
    ///
    /// # Returns
    /// Uptime in seconds, formatted duration and hostname.
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

    /// Formats a duration in seconds as a compact, human-readable string.
    ///
    /// # Arguments
    /// * `total_seconds` - Duration to format.
    ///
    /// # Returns
    /// A string such as `"2d 3h 4m"`, `"3h 4m"` or `"4m"`.
    pub fn format_uptime(total_seconds: u64) -> String {
        let days = total_seconds / SECONDS_PER_DAY;
        let hours = (total_seconds % SECONDS_PER_DAY) / SECONDS_PER_HOUR;
        let minutes = (total_seconds % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE;

        match (days, hours) {
            (d, h) if d > 0 => format!("{d}d {h}h {minutes}m"),
            (0, h) if h > 0 => format!("{h}h {minutes}m"),
            _ => format!("{minutes}m"),
        }
    }
}
