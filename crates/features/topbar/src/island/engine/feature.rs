//! The single feature trait and shared capsule dimensions.

/// Compact capsule width for standard features.
pub const CAPSULE_WIDTH: i32 = 180;
/// Expanded capsule width used by the media player.
pub const PLAYER_CAPSULE_WIDTH: i32 = 200;
/// Standard capsule height.
pub const CAPSULE_HEIGHT: i32 = 30;

use super::context::IslandCtx;

/// The one trait every island feature implements.
pub trait IslandFeature {
    /// Stable identifier, also used as the IPC trigger topic.
    fn id(&self) -> &'static str;

    /// Higher priority wins arbitration.
    fn priority(&self) -> u32 {
        50
    }

    fn size(&self) -> (i32, i32) {
        (CAPSULE_WIDTH, CAPSULE_HEIGHT)
    }

    fn hover_keep(&self) -> bool {
        false
    }

    fn capsule_class(&self) -> Option<&'static str> {
        None
    }

    /// Persistent state (media playing, popover open, recording, ...).
    fn is_alive(&self) -> bool {
        false
    }

    /// An open popover locks the capsule until it closes.
    fn is_popover_open(&self) -> bool {
        false
    }

    /// Builds the content widget and wires all handlers/listeners.
    fn build(&mut self, ctx: &IslandCtx) -> gtk4::Widget;

    fn tick(&mut self, _ctx: &IslandCtx) {}
    fn on_show(&mut self) {}
    fn on_hide(&mut self) {}
    fn on_click(&mut self) {}
    fn open_badge(&mut self) {
        self.on_click();
    }
}
