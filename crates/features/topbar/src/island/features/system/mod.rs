//! System Features for the Dynamic Island.
//!
//! Groups every hardware / desktop status indicator feature:
//! - [`volume::VolumeFeature`]: audio output volume & mute changes.
//! - [`brightness::BrightnessFeature`]: display backlight changes.
//! - [`network::NetworkFeature`]: Wi-Fi / Ethernet connectivity.
//! - [`bluetooth::BluetoothFeature`]: connected Bluetooth devices.
//! - [`battery::BatteryFeature`]: charger and low-battery notifications.

pub mod battery;
pub mod bluetooth;
pub mod brightness;
pub mod network;
pub mod volume;

pub use battery::BatteryFeature;
pub use bluetooth::BluetoothFeature;
pub use brightness::BrightnessFeature;
pub use network::NetworkFeature;
pub use volume::VolumeFeature;

// Registration lives in exactly one place: `crate::island::registry`.
