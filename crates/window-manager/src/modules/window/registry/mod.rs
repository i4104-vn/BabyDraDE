//! Z-ordered registry of managed windows.


pub(crate) use crate::modules::window::models::registry::WindowRegistry;

#[cfg(test)]
mod tests {
    use super::*;
    use smithay::desktop::{Space, Window};

    #[test]
    fn active_window_is_none_for_an_empty_registry_and_space() {
        let registry = WindowRegistry::new(|_| false);
        let space: Space<Window> = Space::default();
        assert!(registry.active_window(&space).is_none());
    }
}
pub(crate) mod metadata;

pub(crate) mod cursor;

pub(crate) mod cursor_impl;
