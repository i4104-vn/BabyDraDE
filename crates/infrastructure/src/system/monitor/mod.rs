//! System monitor snapshot structure

use super::SystemService;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct MonitorSnapshot {
    pub cpu_percent: f64,
    pub ram_percent: f64,
    pub ram_used_gb: f64,
    pub ram_total_gb: f64,
    pub gpu_percent: f64,
}

pub fn get_monitor_snapshot(sys: &mut SystemService) -> MonitorSnapshot {
    let snap = sys.snapshot();
    let cpu_pct = snap.cpu.usage_percent as f64;
    let used_gb = snap.memory.used_mb() as f64 / 1024.0;
    let total_gb = snap.memory.total_mb() as f64 / 1024.0;
    let ram_pct = snap.memory.used_percent as f64;

    MonitorSnapshot {
        cpu_percent: cpu_pct,
        ram_percent: ram_pct,
        ram_used_gb: used_gb,
        ram_total_gb: total_gb,
        gpu_percent: 0.0,
    }
}
