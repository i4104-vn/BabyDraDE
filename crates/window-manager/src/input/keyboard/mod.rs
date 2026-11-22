//! Keyboard input processing, focus handling, and global hotkeys.

pub(crate) mod focus;
pub(crate) mod shortcuts;

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
/// Alt+Tab window switching, and Super+Enter for fullscreen), and forwards non-intercepted
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
    let mut alt_tab_hold: Option<i32> = None;

    keyboard.input::<(), _>(
        state,
        event.key_code(),
        event.state(),
        serial,
        time,
        |state, modifiers, handle| {
            let sym = handle.modified_sym().into();
            let switcher_active = state.alt_tab_switcher.is_some();

            // The launcher owns keyboard input while open: Up/Down move the
            // selection (held keys auto-repeat), Return launches, Escape closes.
            if state.launcher.is_visible() {
                let arrow = match sym {
                    keysyms::KEY_Up => Some(-1),
                    keysyms::KEY_Down => Some(1),
                    _ => None,
                };
                if let Some(direction) = arrow {
                    match event.state() {
                        KeyState::Pressed => {
                            if state.launcher.arrow_pressed(direction) {
                                state.launcher.move_selection(direction);
                            }
                        }
                        KeyState::Released => state.launcher.arrow_released(direction),
                    }
                    return FilterResult::Intercept(());
                }

                if event.state() == KeyState::Pressed {
                    match sym {
                        keysyms::KEY_Return | keysyms::KEY_KP_Enter => {
                            state.launcher.activate();
                            return FilterResult::Intercept(());
                        }
                        keysyms::KEY_Escape => {
                            state.launcher.close();
                            return FilterResult::Intercept(());
                        }
                        _ => {}
                    }
                }
            }

            // Alt+Tab holds are tracked here so the switcher can auto-repeat
            // even on backends that do not generate key-repeat events.
            let is_alt_tab =
                modifiers.alt && (sym == keysyms::KEY_Tab || sym == keysyms::KEY_ISO_Left_Tab);
            if is_alt_tab {
                let backwards = modifiers.shift || sym == keysyms::KEY_ISO_Left_Tab;
                let direction = if backwards { -1 } else { 1 };
                match event.state() {
                    KeyState::Pressed => {
                        if let Some(switcher) = state.alt_tab_switcher.as_mut() {
                            if switcher.tab_pressed(direction) {
                                state.cycle_alt_tab(backwards);
                            }
                        } else if let Some(action) = check_key_shortcut(modifiers, sym) {
                            action_to_execute = Some(action);
                            alt_tab_hold = Some(direction);
                        }
                    }
                    KeyState::Released => {
                        if let Some(switcher) = state.alt_tab_switcher.as_mut() {
                            switcher.tab_released(direction);
                        }
                    }
                }
                return FilterResult::Intercept(());
            }

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
                && (!modifiers.alt || sym == keysyms::KEY_Alt_L || sym == keysyms::KEY_Alt_R)
            {
                action_to_execute = Some(KeyboardAction::AltTabCommit);
            }

            if event.state() == KeyState::Pressed {
                if let Some(action) = check_key_shortcut(modifiers, sym) {
                    action_to_execute = Some(action);
                    return FilterResult::Intercept(());
                }

                // Configurable shortcuts use the layout-independent raw keysym
                // so Shift does not change letter matching.
                if let Some(raw_sym) = handle.raw_latin_sym_or_raw_current_sym() {
                    if let Some(command) = crate::keymap::matcher::command_for(
                        modifiers,
                        raw_sym.into(),
                        crate::keymap::shortcuts(),
                    ) {
                        crate::keymap::spawn_command(command);
                        return FilterResult::Intercept(());
                    }
                }
            }

            FilterResult::Forward
        },
    );

    if let Some(action) = action_to_execute {
        execute_action(state, action, serial);
    }
    if let Some(direction) = alt_tab_hold {
        if let Some(switcher) = state.alt_tab_switcher.as_mut() {
            switcher.tab_pressed(direction);
        }
    }
}
