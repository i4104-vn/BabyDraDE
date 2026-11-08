pub mod hardware;
pub mod host;
pub mod network;
pub mod resources;

pub use hardware::{BacklightInfo, BacklightReader, BatteryInfo, BatteryReader, BatteryStatus, ThermalInfo, ThermalReader, ThermalZone};
pub use host::{UptimeInfo, UptimeReader};
pub use network::{InterfaceStats, NetworkInfo, NetworkReader};
pub use resources::{CpuInfo, CpuReader, DiskInfo, DiskReader, MemoryInfo, MemoryReader};

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
