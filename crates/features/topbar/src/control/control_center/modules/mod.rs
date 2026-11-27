//! Control Center UI sub-modules

mod device;
mod network;

pub use device::{backlight, clean, storage, volume};
pub use network::{bluetooth, vpn, wifi};

pub use crate::control::control_center::sections::{header, toggle_grid};
