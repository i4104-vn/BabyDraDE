//! Input event context state shared between handlers.

use smithay::backend::input::KeyState;
use smithay::input::keyboard::ModifiersState;

use common::KeyboardAction;

// ----- shared context -------------------------------------------------

/// Bundles the state and event data needed by the key-family handlers.
pub(crate) struct KeyContext<'a, C> {
    pub(crate) state: &'a mut C,
    pub(crate) key_state: KeyState,
    pub(crate) modifiers: &'a ModifiersState,
    pub(crate) sym: u32,
    pub(crate) action: &'a mut Option<KeyboardAction>,
    pub(crate) alt_tab_hold: &'a mut Option<i32>,
    pub(crate) switcher_active: bool,
}

// ----- pointer button context -----------------------------------------

/// Pointer button event context.
pub(crate) struct ButtonContext<'a, C> {
    pub(crate) state: &'a mut C,
    pub(crate) serial: smithay::utils::Serial,
    pub(crate) button_code: u32,
    pub(crate) time: u32,
}
