/// Backlight brightness control.
pub mod backlight;
/// Battery state reader.
pub mod battery;
/// Thermal zone and hwmon reader.
pub mod thermal;

pub use backlight::{BacklightInfo, BacklightReader};
pub use battery::{BatteryInfo, BatteryReader, BatteryStatus};
pub use thermal::{ThermalInfo, ThermalReader, ThermalZone};
