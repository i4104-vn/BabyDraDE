//! Keyboard input processing, focus handling, and global hotkeys.

pub(crate) mod focus;
pub(crate) mod shortcuts;

pub(crate) use focus::cycle_window_focus;

use smithay::{
    backend::input::{Event, InputBackend, KeyState, KeyboardKeyEvent},
    input::keyboard::{keysyms, xkb, FilterResult},
    utils::SERIAL_COUNTER,
};

use super::InputContext;
use shortcuts::{check_key_shortcut, execute_action, KeyboardAction};

/// Dispatches and processes an incoming keyboard event from the input backend.
///
/// Filters global window manager shortcuts (such as Super+Q to close window,
/// Alt+Tab window switching, and Super+Enter for fullscreen), and forwards non-intercepted
/// key events to the Wayland surface holding keyboard focus.
pub(crate) fn handle_keyboard<B: InputBackend, K: KeyboardKeyEvent<B>>(
    state: &mut impl InputContext,
    event: K,
) {
    let serial = SERIAL_COUNTER.next_serial();
    let time = Event::time_msec(&event);

    let mut action_to_execute = None;
    let mut alt_tab_hold: Option<i32> = None;

    state.keyboard_input(
        event.key_code(),
        event.state(),
        serial,
        time,
        |state, modifiers, handle| {
            let sym = handle.modified_sym().into();
            let switcher_active = state.switcher_is_active();

            if state.launcher_is_visible() {
                let search_active = state.launcher_search_is_active();

                let arrow = match sym {
                    keysyms::KEY_Up => Some(-1),
                    keysyms::KEY_Down => Some(1),
                    _ => None,
                };
                if let Some(direction) = arrow {
                    match event.state() {
                        KeyState::Pressed => {
                            if search_active {
                                state.launcher_search_move(direction);
                            } else if state.launcher_arrow_pressed(direction) {
                                state.launcher_move_selection(direction);
                            }
                        }
                        KeyState::Released => {
                            if !search_active {
                                state.launcher_arrow_released(direction);
                            }
                        }
                    }
                    return FilterResult::Intercept(());
                }

                if event.state() == KeyState::Pressed {
                    match sym {
                        keysyms::KEY_Return | keysyms::KEY_KP_Enter => {
                            state.launcher_activate();
                            return FilterResult::Intercept(());
                        }
                        keysyms::KEY_Escape => {
                            if !state.launcher_search_escape() {
                                state.launcher_close();
                            }
                            return FilterResult::Intercept(());
                        }
                        keysyms::KEY_BackSpace => {
                            let search = state.launcher_search_text_mut();
                            if modifiers.ctrl {
                                search.delete_word();
                            } else {
                                search.backspace();
                            }
                            return FilterResult::Intercept(());
                        }
                        keysyms::KEY_Delete | keysyms::KEY_KP_Delete if search_active => {
                            state.launcher_search_text_mut().delete();
                            return FilterResult::Intercept(());
                        }
                        keysyms::KEY_Home | keysyms::KEY_KP_Home if search_active => {
                            state.launcher_search_text_mut().home(modifiers.shift);
                            return FilterResult::Intercept(());
                        }
                        keysyms::KEY_End | keysyms::KEY_KP_End if search_active => {
                            state.launcher_search_text_mut().end(modifiers.shift);
                            return FilterResult::Intercept(());
                        }
                        keysyms::KEY_a | keysyms::KEY_A
                            if search_active
                                && modifiers.ctrl
                                && !modifiers.logo
                                && !modifiers.alt =>
                        {
                            state.launcher_search_text_mut().select_all();
                            return FilterResult::Intercept(());
                        }
                        _ => {}
                    }

                    // Caret movement. Ctrl jumps by word, Shift extends the
                    // selection. The search box owns horizontal arrows; the
                    // result list keeps Up/Down.
                    if search_active && !modifiers.logo && !modifiers.alt {
                        let caret_move = match sym {
                            keysyms::KEY_Left | keysyms::KEY_KP_Left => Some((-1, modifiers.ctrl)),
                            keysyms::KEY_Right | keysyms::KEY_KP_Right => Some((1, modifiers.ctrl)),
                            _ => None,
                        };
                        if let Some((direction, word)) = caret_move {
                            state.launcher_search_text_mut().move_caret(
                                direction,
                                word,
                                modifiers.shift,
                            );
                            return FilterResult::Intercept(());
                        }
                    }

                    // A bare printable key inserts at the caret. Modifier
                    // combinations are left to the shortcut pipeline.
                    if !modifiers.logo && !modifiers.ctrl && !modifiers.alt {
                        let text = xkb::keysym_to_utf8(handle.modified_sym());
                        if let Some(character) = text.chars().next() {
                            if !character.is_control() {
                                state
                                    .launcher_search_text_mut()
                                    .insert(&character.to_string());
                                return FilterResult::Intercept(());
                            }
                        }
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
                        if state.switcher_is_active() {
                            if state.switcher_tab_pressed(direction) {
                                state.cycle_alt_tab(backwards);
                            }
                        } else if let Some(action) = check_key_shortcut(modifiers, sym) {
                            action_to_execute = Some(action);
                            alt_tab_hold = Some(direction);
                        }
                    }
                    KeyState::Released => {
                        state.switcher_tab_released(direction);
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
                    if let Some(command) = crate::adapters::input::keymap::matcher::command_for(
                        &*state,
                        modifiers,
                        raw_sym.into(),
                    ) {
                        crate::adapters::input::keymap::spawn_command(&*state, command);
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
        state.switcher_tab_pressed(direction);
    }
}
