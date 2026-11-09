//! Extensible Island Feature Trait and Registry

pub mod default;
pub mod media;
pub mod recording;

pub use default::DefaultFeature;
pub use media::MediaFeature;
pub use recording::RecordingFeature;

/// Trait defining a pluggable feature for the Dynamic Island.
///
/// Features can be easily added (e.g. Media Player, Screen Recorder, Clipboard preview,
/// Notifications, Pomodoro Timer) and will automatically be arbitrated based on priority.
pub trait IslandFeature {
    /// Unique identifier
    fn id(&self) -> &'static str;

    /// Priority order (higher priority preempts lower priority when active)
    fn priority(&self) -> u32;

    /// Whether this feature is currently active and wants to be displayed
    fn is_active(&self) -> bool;

    /// The visual widget placed inside the Dynamic Island capsule
    fn widget(&self) -> gtk4::Widget;

    /// Optional details popover when the user clicks the island
    fn details_popover_content(&self) -> Option<gtk4::Widget> {
        None
    }
}

