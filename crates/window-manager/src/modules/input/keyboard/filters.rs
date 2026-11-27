//! Keyboard event filters: shared event context, launcher keys, switcher keys,
//! and global or configurable shortcut keys.

use smithay::{
    backend::input::KeyState,
    input::keyboard::{keysyms, xkb, KeysymHandle},
};

use common::ShortcutTarget;

pub(crate) use crate::modules::input::models::KeyContext;
use crate::modules::input::{
    keyboard::shortcuts::{check_key_shortcut, KeyboardAction},
    keymap::{
        matcher::{command_for, to_mods},
        spawn_command,
    },
    InputContext,
};
use crate::shared::ports::{CaretSelection, CaretUnit};

/// Handles keys while the launcher is visible.
///
/// # Arguments
/// * `ctx` - Shared keyboard context.
/// * `handle` - Keysym handle used for printable character conversion.
///
/// # Returns
/// `true` when the key was consumed by the launcher.
pub(crate) fn handle_launcher<C: InputContext>(
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
            let unit = if word {
                CaretUnit::Word
            } else {
                CaretUnit::Character
            };
            let selection = if modifiers.shift {
                CaretSelection::Extend
            } else {
                CaretSelection::Move
            };
            ctx.state
                .launcher_search_text_mut()
                .move_caret(direction, unit, selection);
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

/// Handles switcher navigation, cancellation, and commit keys.
///
/// # Arguments
/// * `ctx` - Shared keyboard context.
///
/// # Returns
/// `true` when the key was consumed by the switcher.
pub(crate) fn handle_switcher<C: InputContext>(ctx: &mut KeyContext<'_, C>) -> bool {
    let sym = ctx.sym;
    let modifiers = ctx.modifiers;
    let switcher_active = ctx.switcher_active;

    // Alt+Tab holds are tracked here so the switcher can auto-repeat
    // even on backends that do not generate key-repeat events.
    let is_alt_tab = modifiers.alt && (sym == keysyms::KEY_Tab || sym == keysyms::KEY_ISO_Left_Tab);
    if is_alt_tab {
        let backwards = modifiers.shift || sym == keysyms::KEY_ISO_Left_Tab;
        let direction = if backwards { -1 } else { 1 };
        match ctx.key_state {
            KeyState::Pressed => {
                if ctx.state.switcher_is_active() {
                    if ctx.state.switcher_tab_pressed(direction) {
                        ctx.state.cycle_alt_tab(backwards);
                    }
                } else if let Some(action) = check_key_shortcut(modifiers, sym) {
                    *ctx.action = Some(action);
                    *ctx.alt_tab_hold = Some(direction);
                }
            }
            KeyState::Released => {
                ctx.state.switcher_tab_released(direction);
            }
        }
        return true;
    }

    if switcher_active
        && ctx.key_state == KeyState::Released
        && (sym == keysyms::KEY_Tab || sym == keysyms::KEY_ISO_Left_Tab)
    {
        return true;
    }

    if switcher_active
        && ctx.key_state == KeyState::Pressed
        && sym == keysyms::KEY_Escape
        && !modifiers.logo
    {
        *ctx.action = Some(KeyboardAction::AltTabCancel);
        return true;
    }

    if switcher_active
        && ctx.key_state == KeyState::Released
        && (!modifiers.alt || sym == keysyms::KEY_Alt_L || sym == keysyms::KEY_Alt_R)
    {
        *ctx.action = Some(KeyboardAction::AltTabCommit);
    }

    false
}

/// Handles global shortcuts and layout-independent configurable commands.
///
/// # Arguments
/// * `ctx` - Shared keyboard context.
/// * `handle` - Keysym handle used for raw configurable-command matching.
///
/// # Returns
/// `true` when the key was consumed by a shortcut.
pub(crate) fn handle_shortcuts<C: InputContext>(
    ctx: &mut KeyContext<'_, C>,
    handle: &KeysymHandle<'_>,
) -> bool {
    if ctx.key_state != KeyState::Pressed {
        return false;
    }

    if let Some(action) = check_key_shortcut(ctx.modifiers, ctx.sym) {
        *ctx.action = Some(action);
        return true;
    }

    // Runtime registrations and configurable shortcuts use the
    // layout-independent raw keysym so Shift does not change letter matching.
    let Some(raw) = handle.raw_latin_sym_or_raw_current_sym() else {
        return false;
    };
    let raw_sym: u32 = raw.into();

    match ctx
        .state
        .registered_shortcut(to_mods(ctx.modifiers), raw_sym)
    {
        Some(ShortcutTarget::Action(action)) => {
            *ctx.action = Some(action);
            return true;
        }
        Some(ShortcutTarget::Command(command)) => {
            spawn_command(&*ctx.state, command);
            return true;
        }
        None => {}
    }

    if let Some(command) = command_for(&*ctx.state, ctx.modifiers, raw_sym) {
        spawn_command(&*ctx.state, command);
        return true;
    }

    false
}
