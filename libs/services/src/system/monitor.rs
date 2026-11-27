//! System monitor snapshot structure

use super::SystemService;

/// MiB in one GiB; converts the MiB readings from `SystemService` into GiB.
const MIB_PER_GIB: f64 = 1024.0;

/// CPU, memory and GPU usage sampled for the system monitor.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MonitorSnapshot {
    pub cpu_percent: f64,
    pub ram_percent: f64,
    pub ram_used_gb: f64,
    pub ram_total_gb: f64,
    pub gpu_percent: f64,
}

/// Refreshes `sys` and samples current resource usage.
///
/// GPU usage is reported as `0.0` until a source is wired up.
///
/// # Arguments
/// * `sys` - System service used to refresh the readings.
pub fn get_monitor_snapshot(sys: &mut SystemService) -> MonitorSnapshot {
    let snap = sys.snapshot();
    let cpu_pct = snap.cpu.usage_percent as f64;
    let used_gb = snap.memory.used_mb() as f64 / MIB_PER_GIB;
    let total_gb = snap.memory.total_mb() as f64 / MIB_PER_GIB;
    let ram_pct = snap.memory.used_percent as f64;

    MonitorSnapshot {
        cpu_percent: cpu_pct,
        ram_percent: ram_pct,
        ram_used_gb: used_gb,
        ram_total_gb: total_gb,
        gpu_percent: 0.0,
    }
}
