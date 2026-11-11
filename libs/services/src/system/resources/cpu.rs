use std::fs;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CpuInfo {
    pub usage_percent: f32,
    pub core_count: usize,
    pub cur_freq_mhz: Option<u32>,
    pub per_core_usage: Vec<f32>,
}

#[derive(Debug, Clone, Default)]
struct CpuStatSnapshot {
    idle: u64,
    total: u64,
}

#[derive(Debug, Clone)]
pub struct CpuReader {
    last_overall: Option<CpuStatSnapshot>,
    last_per_core: Vec<CpuStatSnapshot>,
    core_count: usize,
}

impl Default for CpuReader {
    fn default() -> Self {
        Self::new()
    }
}

impl CpuReader {
    pub fn new() -> Self {
        let core_count = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or_else(|_| Self::detect_cores_from_proc().unwrap_or(1));

        let mut reader = Self {
            last_overall: None,
            last_per_core: Vec::new(),
            core_count,
        };

        // Initialize snapshot
        let _ = reader.read();
        reader
    }

    pub fn read(&mut self) -> CpuInfo {
        let (overall_stat, per_core_stats) = match Self::parse_proc_stat() {
            Some(res) => res,
            None => {
                return CpuInfo {
                    usage_percent: 0.0,
                    core_count: self.core_count,
                    cur_freq_mhz: Self::read_cur_freq_mhz(),
                    per_core_usage: Vec::new(),
                };
            }
        };

        // Calculate overall usage delta
        let usage_percent = if let Some(ref prev) = self.last_overall {
            Self::calc_usage_percent(prev, &overall_stat)
        } else {
            0.0
        };
        self.last_overall = Some(overall_stat);

        // Calculate per-core usage deltas
        let mut per_core_usage = Vec::with_capacity(per_core_stats.len());
        for (i, current) in per_core_stats.iter().enumerate() {
            let percent = if let Some(prev) = self.last_per_core.get(i) {
                Self::calc_usage_percent(prev, current)
            } else {
                0.0
            };
            per_core_usage.push(percent);
        }
        self.last_per_core = per_core_stats;

        CpuInfo {
            usage_percent,
            core_count: self.core_count,
            cur_freq_mhz: Self::read_cur_freq_mhz(),
            per_core_usage,
        }
    }

    fn calc_usage_percent(prev: &CpuStatSnapshot, current: &CpuStatSnapshot) -> f32 {
        let total_delta = current.total.saturating_sub(prev.total);
        let idle_delta = current.idle.saturating_sub(prev.idle);

        if total_delta == 0 {
            0.0
        } else {
            let active_delta = total_delta.saturating_sub(idle_delta);
            ((active_delta as f64 / total_delta as f64) * 100.0) as f32
        }
        .clamp(0.0, 100.0)
    }

    fn parse_proc_stat() -> Option<(CpuStatSnapshot, Vec<CpuStatSnapshot>)> {
        let content = fs::read_to_string("/proc/stat").ok()?;
        let mut overall: Option<CpuStatSnapshot> = None;
        let mut per_core = Vec::new();

        for line in content.lines() {
            let first_word = line.split_whitespace().next().unwrap_or("");
            match first_word {
                "cpu" => {
                    overall = Self::parse_cpu_line(line);
                }
                word if word.starts_with("cpu")
                    && word[3..].chars().all(|c| c.is_ascii_digit()) =>
                {
                    if let Some(snapshot) = Self::parse_cpu_line(line) {
                        per_core.push(snapshot);
                    }
                }
                _ => {}
            }
        }

        overall.map(|ov| (ov, per_core))
    }

    fn parse_cpu_line(line: &str) -> Option<CpuStatSnapshot> {
        let mut parts = line.split_whitespace();
        parts.next()?; // Skip "cpu" or "cpuN"

        let mut values = Vec::new();
        for val_str in parts {
            if let Ok(v) = val_str.parse::<u64>() {
                values.push(v);
            }
        }

        if values.len() < 4 {
            return None;
        }

        let user = values.first().copied().unwrap_or(0);
        let nice = values.get(1).copied().unwrap_or(0);
        let system = values.get(2).copied().unwrap_or(0);
        let idle = values.get(3).copied().unwrap_or(0);
        let iowait = values.get(4).copied().unwrap_or(0);
        let irq = values.get(5).copied().unwrap_or(0);
        let softirq = values.get(6).copied().unwrap_or(0);
        let steal = values.get(7).copied().unwrap_or(0);

        let idle_total = idle + iowait;
        let total = user + nice + system + idle_total + irq + softirq + steal;

        Some(CpuStatSnapshot {
            idle: idle_total,
            total,
        })
    }

    fn detect_cores_from_proc() -> Option<usize> {
        let content = fs::read_to_string("/proc/cpuinfo").ok()?;
        let count = content
            .lines()
            .filter(|line| line.starts_with("processor\t") || line.starts_with("processor :"))
            .count();

        if count > 0 {
            Some(count)
        } else {
            None
        }
    }

    fn read_cur_freq_mhz() -> Option<u32> {
        let khz_str = fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq")
            .or_else(|_| {
                fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_cur_freq")
            })
            .ok()?;

        let khz: u32 = khz_str.trim().parse().ok()?;
        Some(khz / 1000)
    }
}
