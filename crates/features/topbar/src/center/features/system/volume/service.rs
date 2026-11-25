//! Data source for the volume indicator (read by `crate::center::poller`).

pub use services::system::volume::get_volume_state;

/// Snapshot of audio output state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VolumeState {
    pub volume: f64,
    pub muted: bool,
}

/// One poll of the volume service.
pub fn read() -> VolumeState {
    let (volume, muted) = get_volume_state();
    VolumeState { volume, muted }
}
