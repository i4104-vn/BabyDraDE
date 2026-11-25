//! Recording configuration models.
//!
//! Upstream imported these from `babydra_core::models::recording`; the
//! topbar crate owns its own copies so the recorder UI stays independent of
//! the recorder daemon implementation.

/// Area geometry selected with `slurp`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RecordingArea {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

/// Which part of the screen should be captured.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum RecordingMode {
    #[default]
    Fullscreen,
    SingleOutput(String),
    Area(RecordingArea),
    Window(String),
}

/// User-configurable recording options.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordingConfig {
    pub mode: RecordingMode,
    pub resolution: Option<(u32, u32)>,
    pub framerate: u32,
    pub format: String,
    pub hdr: bool,
    pub audio: bool,
    pub audio_device: Option<String>,
    pub codec: Option<String>,
}

impl Default for RecordingConfig {
    fn default() -> Self {
        Self {
            mode: RecordingMode::Fullscreen,
            resolution: None,
            framerate: 60,
            format: "mp4".to_string(),
            hdr: false,
            audio: false,
            audio_device: None,
            codec: None,
        }
    }
}
