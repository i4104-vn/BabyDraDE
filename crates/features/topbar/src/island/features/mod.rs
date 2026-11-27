//! Dynamic Island features.
//!
//! Mirrors the complete feature set of `babydra-island`: clipboard, default
//! (idle), media player, power, recording and the system indicators.

pub mod clipboard;
pub mod default;
pub mod media;
pub mod power;
pub mod recording;
pub mod system;

pub use clipboard::ClipboardFeature;
pub use default::DefaultFeature;
pub use media::MediaFeature;
pub use power::PowerFeature;
pub use recording::RecordingFeature;
pub use system::{
    BatteryFeature, BluetoothFeature, BrightnessFeature, NetworkFeature, VolumeFeature,
};

pub use crate::island::engine::IslandFeature;
