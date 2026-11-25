//! The single place where the island's feature set is assembled.
//!
//! To add a feature: implement `IslandFeature` in `features/`, export it from
//! `features/mod.rs`, and append one `register_feature` line here. No other
//! wiring (handles, triggers, popovers) is needed.

use crate::center::engine::IslandEngine;
use crate::center::features::{
    clipboard::ClipboardFeature,
    power::PowerFeature,
    recording::RecordingFeature,
    system::{BatteryFeature, BluetoothFeature, BrightnessFeature, NetworkFeature, VolumeFeature},
    MediaFeature,
};

/// Registers every built-in feature in upstream priority/registration order.
pub fn register_all(engine: &mut IslandEngine) {
    engine.register_feature(PowerFeature::new());
    engine.register_feature(VolumeFeature::new());
    engine.register_feature(BrightnessFeature::new());
    engine.register_feature(NetworkFeature::new());
    engine.register_feature(BluetoothFeature::new());
    engine.register_feature(BatteryFeature::new());
    engine.register_feature(ClipboardFeature::new());
    engine.register_feature(RecordingFeature::new());
    engine.register_feature(MediaFeature::new());
}
