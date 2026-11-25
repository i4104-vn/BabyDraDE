//! Global and configurable shortcut key handling.

use common::ShortcutTarget;
use smithay::{backend::input::KeyState, input::keyboard::KeysymHandle};

use crate::adapters::input::{
    keyboard::shortcuts::check_key_shortcut,
    keymap::{
        matcher::{command_for, to_mods},
        spawn_command,
    },
    InputContext,
};

use super::models::KeyContext;

/// Handles global shortcuts and layout-independent configurable commands.
///
/// # Arguments
/// * `ctx` - Shared keyboard context.
/// * `handle` - Keysym handle used for raw configurable-command matching.
///
/// # Returns
/// `true` when the key was consumed by a shortcut.
pub(super) fn handle<C: InputContext>(
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

    match ctx.state.registered_shortcut(to_mods(ctx.modifiers), raw_sym) {
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
