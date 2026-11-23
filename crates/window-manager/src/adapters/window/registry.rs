//! Z-ordered registry of managed windows.

use std::ops::{Deref, DerefMut};

use smithay::{
    desktop::{Space, Window},
    utils::IsAlive,
    wayland::shell::xdg::ToplevelSurface,
};

use crate::adapters::compositor::state::window::is_window_exiting;

/// Tracks the windows managed by the compositor, in bottom-to-top Z order.
///
/// Derefs to the underlying `Vec<Window>` so existing iteration and mutation
/// call sites keep working without exposing the vector's ownership.
pub(crate) struct WindowRegistry {
    pub(crate) entries: Vec<Window>,
}

impl WindowRegistry {
    pub(crate) fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Returns the topmost (most recently raised / focused) window, if any.
    ///
    /// Excludes windows that are currently playing their closing animation.
    pub(crate) fn active_window(&self, space: &Space<Window>) -> Option<Window> {
        self.entries
            .iter()
            .rev()
            .filter(|w| !is_window_exiting(w))
            .find(|w| w.alive())
            .cloned()
            .or_else(|| {
                space
                    .elements()
                    .rev()
                    .find(|w| !is_window_exiting(w))
                    .cloned()
            })
    }

    /// Raises a window to the top of the internal Z-order list.
    pub(crate) fn raise(&mut self, window: &Window) {
        if let Some(pos) = self.entries.iter().position(|w| w == window) {
            let win = self.entries.remove(pos);
            self.entries.push(win);
        }
    }

    /// Finds a window associated with an XDG toplevel surface.
    pub(crate) fn find_window_for_surface(
        &self,
        surface: &ToplevelSurface,
        space: &Space<Window>,
    ) -> Option<Window> {
        self.entries
            .iter()
            .find(|w| w.toplevel().map(|t| t == surface).unwrap_or(false))
            .cloned()
            .or_else(|| {
                space
                    .elements()
                    .find(|w| w.toplevel().map(|t| t == surface).unwrap_or(false))
                    .cloned()
            })
    }
}

impl Deref for WindowRegistry {
    type Target = Vec<Window>;

    fn deref(&self) -> &Self::Target {
        &self.entries
    }
}

impl DerefMut for WindowRegistry {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entries
    }
}
