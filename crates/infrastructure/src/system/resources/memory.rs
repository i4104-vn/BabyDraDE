use std::fs;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct MemoryInfo {
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
    pub buffers_bytes: u64,
    pub cached_bytes: u64,
    pub swap_total_bytes: u64,
    pub swap_used_bytes: u64,
    pub swap_free_bytes: u64,
    pub used_percent: f32,
    pub swap_percent: f32,
}

impl MemoryInfo {
    pub fn total_mb(&self) -> u64 {
        self.total_bytes / (1024 * 1024)
    }

    pub fn used_mb(&self) -> u64 {
        self.used_bytes / (1024 * 1024)
    }

    pub fn available_mb(&self) -> u64 {
        self.available_bytes / (1024 * 1024)
    }

    pub fn swap_total_mb(&self) -> u64 {
        self.swap_total_bytes / (1024 * 1024)
    }

    pub fn swap_used_mb(&self) -> u64 {
        self.swap_used_bytes / (1024 * 1024)
    }
}

#[derive(Debug, Default, Clone)]
pub struct MemoryReader;

impl MemoryReader {
    pub fn new() -> Self {
        Self
    }

    pub fn read(&self) -> MemoryInfo {
        Self::parse_meminfo().unwrap_or_default()
    }

    fn parse_meminfo() -> Option<MemoryInfo> {
        let content = fs::read_to_string("/proc/meminfo").ok()?;

        let mut total_kb = 0u64;
        let mut free_kb = 0u64;
        let mut avail_kb = 0u64;
        let mut buffers_kb = 0u64;
        let mut cached_kb = 0u64;
        let mut swap_total_kb = 0u64;
        let mut swap_free_kb = 0u64;

        for line in content.lines() {
            let mut parts = line.split_whitespace();
            let key = match parts.next() {
                Some(k) => k,
                None => continue,
            };
            let val = match parts.next().and_then(|v| v.parse::<u64>().ok()) {
                Some(v) => v,
                None => continue,
            };

            match key {
                "MemTotal:" => total_kb = val,
                "MemFree:" => free_kb = val,
                "MemAvailable:" => avail_kb = val,
                "Buffers:" => buffers_kb = val,
                "Cached:" => cached_kb = val,
                "SwapTotal:" => swap_total_kb = val,
                "SwapFree:" => swap_free_kb = val,
                _ => {}
            }
        }

        if total_kb == 0 {
            return None;
        }

        // If MemAvailable is not reported by kernel, estimate: free + buffers + cached
        if avail_kb == 0 {
            avail_kb = free_kb + buffers_kb + cached_kb;
        }

        let used_kb = total_kb.saturating_sub(avail_kb);
        let used_percent = (used_kb as f64 / total_kb as f64 * 100.0) as f32;

        let swap_used_kb = swap_total_kb.saturating_sub(swap_free_kb);
        let swap_percent = if swap_total_kb > 0 {
            (swap_used_kb as f64 / swap_total_kb as f64 * 100.0) as f32
        } else {
            0.0
        };

        Some(MemoryInfo {
            total_bytes: total_kb * 1024,
            available_bytes: avail_kb * 1024,
            used_bytes: used_kb * 1024,
            free_bytes: free_kb * 1024,
            buffers_bytes: buffers_kb * 1024,
            cached_bytes: cached_kb * 1024,
            swap_total_bytes: swap_total_kb * 1024,
            swap_used_bytes: swap_used_kb * 1024,
            swap_free_bytes: swap_free_kb * 1024,
            used_percent: used_percent.clamp(0.0, 100.0),
            swap_percent: swap_percent.clamp(0.0, 100.0),
        })
    }
}
