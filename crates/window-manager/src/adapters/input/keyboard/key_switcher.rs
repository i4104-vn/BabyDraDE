//! Alt-Tab switcher key handling.

use smithay::{
    backend::input::KeyState,
    input::keyboard::keysyms,
};

use crate::adapters::input::{
    keyboard::shortcuts::{check_key_shortcut, KeyboardAction},
    InputContext,
};

use super::models::KeyContext;

/// Handles switcher navigation, cancellation, and commit keys.
///
/// # Arguments
/// * `ctx` - Shared keyboard context.
///
/// # Returns
/// `true` when the key was consumed by the switcher.
pub(super) fn handle<C: InputContext>(ctx: &mut KeyContext<'_, C>) -> bool {
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
