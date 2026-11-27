use smithay::desktop::{Space, Window};

use super::*;

#[test]
fn active_window_is_none_for_an_empty_registry_and_space() {
    let registry = WindowRegistry::new(|_| false);
    let space: Space<Window> = Space::default();
    assert!(registry.active_window(&space).is_none());
}
