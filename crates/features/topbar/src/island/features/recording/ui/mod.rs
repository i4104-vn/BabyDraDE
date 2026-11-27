//! UI components for recording feature on Dynamic Island.

mod controls;
pub mod popover;

pub use button::RecordingButtonWidget;
pub use capsule::RecordingCapsuleWidget;
pub use controls::{button, capsule};
pub use popover::RecordingPopover;
