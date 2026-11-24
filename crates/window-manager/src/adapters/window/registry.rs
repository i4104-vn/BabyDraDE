//! Z-ordered registry of managed windows.

use std::ops::{Deref, DerefMut};

use smithay::{
    desktop::{Space, Window},
    utils::IsAlive,
    wayland::shell::xdg::ToplevelSurface,
};

/// Tracks the windows managed by the compositor, in bottom-to-top Z order.
///
/// Derefs to the underlying `Vec<Window>` so existing iteration and mutation
/// call sites keep working without exposing the vector's ownership.
pub(crate) struct WindowRegistry {
    pub(crate) entries: Vec<Window>,
    /// Output space containing the mapped windows.
    pub(crate) space: Space<Window>,
    /// Policy supplied by the compositor to skip windows leaving the screen
    /// (closing or minimizing) when resolving the active window.
    is_exiting: fn(&Window) -> bool,
}

impl WindowRegistry {
    pub(crate) fn new(is_exiting: fn(&Window) -> bool) -> Self {
        Self {
            entries: Vec::new(),
            space: Space::default(),
            is_exiting,
        }
    }

    /// Returns the topmost live window, falling back to the topmost mapped
    /// window in the space.
    ///
    /// This is the single resolver for "which window is active"; callers must
    /// not pick a target by iterating the space themselves. Windows leaving
    /// the screen are excluded in both the registry and the space.
    pub(crate) fn active_window(&self, space: &Space<Window>) -> Option<Window> {
        let is_exiting = self.is_exiting;
        self.entries
            .iter()
            .rev()
            .filter(|w| !is_exiting(w))
            .find(|w| w.alive())
            .cloned()
            .or_else(|| space.elements().rev().find(|w| !is_exiting(w)).cloned())
    }

    /// Raises a window to the top of the internal Z-order list.
    pub(crate) fn raise(&mut self, window: &Window) {
        if let Some(pos) = self.entries.iter().position(|w| w == window) {
            let win = self.entries.remove(pos);
            self.entries.push(win);
        }
    }

    /// Finds a window associated with an XDG toplevel surface.
    pub(crate) fn find_window(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_window_is_none_for_an_empty_registry_and_space() {
        let registry = WindowRegistry::new(|_| false);
        let space: Space<Window> = Space::default();
        assert!(registry.active_window(&space).is_none());
    }
}
