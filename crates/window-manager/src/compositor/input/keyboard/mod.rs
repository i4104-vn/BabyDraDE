//! Keyboard input processing, focus handling, and global hotkeys.

pub mod focus;
pub mod launcher;
pub mod shortcuts;

pub(crate) use focus::cycle_window_focus;

use smithay::{
    backend::input::{Event, InputBackend, KeyState, KeyboardKeyEvent},
    input::keyboard::{keysyms, FilterResult},
    utils::SERIAL_COUNTER,
};

use crate::compositor::state::BabydraState;
use shortcuts::{check_key_shortcut, execute_action, KeyboardAction};

/// Dispatches and processes an incoming keyboard event from the input backend.
///
/// Filters global window manager shortcuts (such as Super+Q to close window,
/// Alt+Tab window switching, and Super+T for terminal), and forwards non-intercepted
/// key events to the Wayland surface holding keyboard focus.
///
/// # Arguments
///
/// * `state` - Mutable reference to [`BabydraState`].
/// * `event` - The keyboard key event providing key code, state (press/release), and timestamp.
pub(crate) fn handle_keyboard<B: InputBackend, K: KeyboardKeyEvent<B>>(
    state: &mut BabydraState,
    event: K,
) {
    let serial = SERIAL_COUNTER.next_serial();
    let time = Event::time_msec(&event);
    let keyboard = state.seat.get_keyboard().unwrap();

    let mut action_to_execute = None;

    keyboard.input::<(), _>(
        state,
        event.key_code(),
        event.state(),
        serial,
        time,
        |state, modifiers, handle| {
            let sym = handle.modified_sym().into();
            let switcher_active = state.alt_tab_switcher.is_some();

            if switcher_active
                && event.state() == KeyState::Released
                && (sym == keysyms::KEY_Tab || sym == keysyms::KEY_ISO_Left_Tab)
            {
                return FilterResult::Intercept(());
            }

            if switcher_active
                && event.state() == KeyState::Pressed
                && sym == keysyms::KEY_Escape
                && !modifiers.logo
            {
                action_to_execute = Some(KeyboardAction::AltTabCancel);
                return FilterResult::Intercept(());
            }

            if switcher_active
                && event.state() == KeyState::Released
                && (!modifiers.alt
                    || sym == keysyms::KEY_Alt_L
                    || sym == keysyms::KEY_Alt_R)
            {
                action_to_execute = Some(KeyboardAction::AltTabCommit);
            }

            if event.state() == KeyState::Pressed {
                if let Some(action) = check_key_shortcut(modifiers, sym) {
                    action_to_execute = Some(action);
                    return FilterResult::Intercept(());
                }
            }

            FilterResult::Forward
        },
    );

    if let Some(action) = action_to_execute {
        execute_action(state, action, serial);
    }
}
