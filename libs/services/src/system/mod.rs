//! Aggregated access to system monitoring: CPU, memory, battery, thermal,
//! disk, network, uptime, volume and window state.
pub mod clipboard;
/// D-Bus application menu discovery and activation.
pub mod dbus_menu;
/// Backlight, battery and thermal sensors.
pub mod hardware;
/// Host uptime information.
pub mod host;
/// Monitor geometry snapshot.
pub mod monitor;
/// Network interface detection and statistics.
pub mod network;
/// CPU, memory and disk usage readers.
pub mod resources;
/// Temperature readings from all kernel sources.
pub mod temperature;
/// Audio volume control.
pub mod volume;
/// VPN detection.
pub mod vpn;
/// App-published keyboard shortcut registration.
pub mod shortcuts;
/// Active window metadata and actions.
pub mod window;

pub use clipboard::{clear_history, history as clipboard_history, store_selection, ClipboardEntry};
pub use dbus_menu::{
    activate_dbus_menu_item, fetch_dbus_menu, fetch_dbus_menu_with_prepared_submenus,
    find_app_menu_service, DbusMenuItem,
};
pub use shortcuts::{read_all, ShortcutClient, ShortcutError, ShortcutTarget};
pub use window::{
    close_window_by_app, get_active_window, get_app_display_name, maximize_active_window,
    minimize_active_window, read_and_clear_window_action, save_active_window, send_window_action,
    toggle_fullscreen_active_window, ActiveWindowInfo,
};

pub use hardware::{
    BacklightInfo, BacklightReader, BatteryInfo, BatteryReader, BatteryStatus, ThermalInfo,
    ThermalReader, ThermalZone,
};
pub use host::{UptimeInfo, UptimeReader};
pub use monitor::{get_monitor_snapshot, MonitorSnapshot};
pub use network::{
    format_speed, get_active_network_info, ActiveNetworkInfo, ActiveNetworkType, InterfaceStats,
    NetworkInfo, NetworkReader,
};
pub use resources::{CpuInfo, CpuReader, DiskInfo, DiskReader, MemoryInfo, MemoryReader};
pub use temperature::{
    get_all_temperatures, get_cpu_temperature, get_important_temperatures, TemperatureReading,
};
pub use volume::{get_active_output_name, get_volume_state, set_muted, set_volume, AudioBackend};
pub use vpn::is_vpn_active;

/// A point-in-time collection of all monitored system metrics.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SystemSnapshot {
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub battery: Option<BatteryInfo>,
    pub backlight: Option<BacklightInfo>,
    pub thermal: ThermalInfo,
    pub disk: Option<DiskInfo>,
    pub network: NetworkInfo,
    pub uptime: UptimeInfo,
}

/// Owns the readers used to collect a [`SystemSnapshot`].
#[derive(Debug, Clone)]
pub struct SystemService {
    pub cpu: CpuReader,
    pub memory: MemoryReader,
    pub battery: BatteryReader,
    pub backlight: BacklightReader,
    pub thermal: ThermalReader,
    pub disk: DiskReader,
    pub network: NetworkReader,
    pub uptime: UptimeReader,
}

impl Default for SystemService {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemService {
    /// Creates a service with fresh readers for every subsystem.
    ///
    /// # Returns
    /// A ready-to-use [`SystemService`].
    pub fn new() -> Self {
        Self {
            cpu: CpuReader::new(),
            memory: MemoryReader::new(),
            battery: BatteryReader::new(),
            backlight: BacklightReader::new(),
            thermal: ThermalReader::new(),
            disk: DiskReader::default(),
            network: NetworkReader::new(),
            uptime: UptimeReader::new(),
        }
    }

    /// Samples every subsystem once into a [`SystemSnapshot`].
    ///
    /// # Returns
    /// A snapshot containing the latest reading from each subsystem.
    pub fn snapshot(&mut self) -> SystemSnapshot {
        SystemSnapshot {
            cpu: self.cpu.read(),
            memory: self.memory.read(),
            battery: self.battery.read(),
            backlight: self.backlight.read(),
            thermal: self.thermal.read(),
            disk: self.disk.read(),
            network: self.network.read(),
            uptime: self.uptime.read(),
        }
    }
}
