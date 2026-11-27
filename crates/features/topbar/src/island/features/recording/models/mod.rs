//! Models for the recording island feature.

pub mod action;
pub mod config;
pub mod state;

pub use action::PopoverActionsContext;
pub use config::{RecordingArea, RecordingConfig, RecordingMode};
pub use state::IslandRecordingState;
