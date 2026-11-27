//! Status bar indicator widgets

pub mod battery;
pub mod network;

pub use battery::create_battery_w;
pub use network::create_network_widget;
