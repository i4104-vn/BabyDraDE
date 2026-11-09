pub mod dbus_menu;
pub mod hardware;
pub mod host;
pub mod monitor;
pub mod network;
pub mod resources;
pub mod volume;
pub mod vpn;
pub mod window;

pub use dbus_menu::{activate_dbus_menu_item, fetch_dbus_menu, find_app_menu_service, DbusMenuItem};
pub use window::{
    close_window_by_app, get_active_window, get_app_display_name, minimize_active_window,
    save_active_window, toggle_fullscreen_active_window, ActiveWindowInfo,
};

pub use hardware::{BacklightInfo, BacklightReader, BatteryInfo, BatteryReader, BatteryStatus, ThermalInfo, ThermalReader, ThermalZone};
pub use host::{UptimeInfo, UptimeReader};
pub use monitor::{get_monitor_snapshot, MonitorSnapshot};
pub use network::{format_speed, get_active_network_info, ActiveNetworkInfo, ActiveNetworkType, InterfaceStats, NetworkInfo, NetworkReader};
pub use resources::{CpuInfo, CpuReader, DiskInfo, DiskReader, MemoryInfo, MemoryReader};
pub use volume::{get_active_output_name, get_volume_state, set_muted, set_volume, AudioBackend};
pub use vpn::is_vpn_active;

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
