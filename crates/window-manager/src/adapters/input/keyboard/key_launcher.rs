//! Launcher key handling (search, caret, and activation).

use smithay::{
    backend::input::KeyState,
    input::keyboard::{keysyms, xkb, KeysymHandle},
};

use crate::adapters::input::InputContext;

use super::models::KeyContext;

/// Handles keys while the launcher is visible.
///
/// # Arguments
/// * `ctx` - Shared keyboard context.
/// * `handle` - Keysym handle used for printable character conversion.
///
/// # Returns
/// `true` when the key was consumed by the launcher.
pub(super) fn handle<C: InputContext>(
    ctx: &mut KeyContext<'_, C>,
    handle: &KeysymHandle<'_>,
) -> bool {
    if !ctx.state.launcher_is_visible() {
        return false;
    }

    let search_active = ctx.state.launcher_search_is_active();
    let sym = ctx.sym;
    let modifiers = ctx.modifiers;

    let arrow = match sym {
        keysyms::KEY_Up => Some(-1),
        keysyms::KEY_Down => Some(1),
        _ => None,
    };
    if let Some(direction) = arrow {
        match ctx.key_state {
            KeyState::Pressed => {
                if search_active {
                    ctx.state.launcher_search_move(direction);
                } else if ctx.state.launcher_arrow_pressed(direction) {
                    ctx.state.launcher_move_selection(direction);
                }
            }
            KeyState::Released => {
                if !search_active {
                    ctx.state.launcher_arrow_released(direction);
                }
            }
        }
        return true;
    }

    if ctx.key_state != KeyState::Pressed {
        return false;
    }

    match sym {
        keysyms::KEY_Return | keysyms::KEY_KP_Enter => {
            ctx.state.launcher_activate();
            return true;
        }
        keysyms::KEY_Escape => {
            if !ctx.state.launcher_search_escape() {
                ctx.state.launcher_close();
            }
            return true;
        }
        keysyms::KEY_BackSpace => {
            let search = ctx.state.launcher_search_text_mut();
            if modifiers.ctrl {
                search.delete_word();
            } else {
                search.backspace();
            }
            return true;
        }
        keysyms::KEY_Delete | keysyms::KEY_KP_Delete if search_active => {
            ctx.state.launcher_search_text_mut().delete();
            return true;
        }
        keysyms::KEY_Home | keysyms::KEY_KP_Home if search_active => {
            ctx.state.launcher_search_text_mut().home(modifiers.shift);
            return true;
        }
        keysyms::KEY_End | keysyms::KEY_KP_End if search_active => {
            ctx.state.launcher_search_text_mut().end(modifiers.shift);
            return true;
        }
        keysyms::KEY_a | keysyms::KEY_A
            if search_active && modifiers.ctrl && !modifiers.logo && !modifiers.alt =>
        {
            ctx.state.launcher_search_text_mut().select_all();
            return true;
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
            ctx.state
                .launcher_search_text_mut()
                .move_caret(direction, word, modifiers.shift);
            return true;
        }
    }

    // A bare printable key inserts at the caret. Modifier
    // combinations are left to the shortcut pipeline.
    if !modifiers.logo && !modifiers.ctrl && !modifiers.alt {
        let text = xkb::keysym_to_utf8(handle.modified_sym());
        if let Some(character) = text.chars().next() {
            if !character.is_control() {
                ctx.state
                    .launcher_search_text_mut()
                    .insert(&character.to_string());
                return true;
            }
        }
    }

    false
}
