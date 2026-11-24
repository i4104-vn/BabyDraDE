//! Shared keyboard event context.

use smithay::{backend::input::KeyState, input::keyboard::ModifiersState};

use crate::adapters::input::keyboard::shortcuts::KeyboardAction;

/// Bundles the state and event data needed by the key-family handlers.
pub(super) struct KeyContext<'a, C> {
    pub(super) state: &'a mut C,
    pub(super) key_state: KeyState,
    pub(super) modifiers: &'a ModifiersState,
    pub(super) sym: u32,
    pub(super) action: &'a mut Option<KeyboardAction>,
    pub(super) alt_tab_hold: &'a mut Option<i32>,
    pub(super) switcher_active: bool,
}
