//! Global keyboard shortcut definitions, matching, and action execution.

use smithay::input::keyboard::{keysyms, ModifiersState};
use smithay::utils::Serial;
use tracing::info;

use super::focus::{cycle_window_focus, focus_window};
use crate::compositor::state::BabydraState;
use crate::shell::lock;

pub(crate) use common::KeyboardAction;

/// Evaluates a pressed keysym against registered global shortcuts.
///
/// # Arguments
///
/// * `modifiers` - The current active keyboard modifiers state (Ctrl, Alt, Shift, Logo/Super).
/// * `sym` - The resolved XKB keysym code of the pressed key.
///
/// # Returns
///
/// Returns `Some(KeyboardAction)` if a global shortcut matched, or `None` if the event should forward to clients.
pub(crate) fn check_key_shortcut(modifiers: &ModifiersState, sym: u32) -> Option<KeyboardAction> {
    // 1. Super + Escape OR Super + Shift + Q OR Ctrl + Alt + Backspace: Quit WM immediately
    if (modifiers.logo && sym == keysyms::KEY_Escape)
        || (modifiers.logo && modifiers.shift && (sym == keysyms::KEY_q || sym == keysyms::KEY_Q))
        || (modifiers.ctrl && modifiers.alt && sym == keysyms::KEY_BackSpace)
    {
        return Some(KeyboardAction::QuitCompositor);
    }

    // Super + Space: toggle the compositor-owned app launcher.
    if modifiers.logo
        && !modifiers.shift
        && !modifiers.ctrl
        && !modifiers.alt
        && sym == keysyms::KEY_space
    {
        return Some(KeyboardAction::ToggleLauncher);
    }

    // Super + V: open the clipboard history picker. Plain Ctrl+V stays with
    // the focused application so normal copy/paste is never broken.
    if modifiers.logo
        && !modifiers.shift
        && !modifiers.ctrl
        && !modifiers.alt
        && sym == keysyms::KEY_v
    {
        return Some(KeyboardAction::ClipboardPicker);
    }

    // Super + Shift + S: capture the screen. The configurable shortcut list
    // cannot remap this compositor action.
    if modifiers.logo
        && modifiers.shift
        && !modifiers.ctrl
        && !modifiers.alt
        && (sym == keysyms::KEY_s || sym == keysyms::KEY_S)
    {
        return Some(KeyboardAction::Screenshot);
    }

    // 2. Super + Q or Alt + F4: Close active window
    if (modifiers.logo && !modifiers.shift && (sym == keysyms::KEY_q || sym == keysyms::KEY_Q))
        || (modifiers.alt && sym == keysyms::KEY_F4)
    {
        return Some(KeyboardAction::CloseActiveWindow);
    }

    // 3. Super + L: Lock the current session
    if modifiers.logo && !modifiers.shift && (sym == keysyms::KEY_l || sym == keysyms::KEY_L) {
        return Some(KeyboardAction::LockScreen);
    }

    // 4. Alt + Tab: choose a window, then focus it when Alt is released
    if modifiers.alt && (sym == keysyms::KEY_Tab || sym == keysyms::KEY_ISO_Left_Tab) {
        let backwards = modifiers.shift || sym == keysyms::KEY_ISO_Left_Tab;
        return Some(KeyboardAction::AltTabStep { backwards });
    }

    // 5. Super (Win) + F5: Hot reload configuration from disk
    if modifiers.logo && sym == keysyms::KEY_F5 {
        return Some(KeyboardAction::ReloadConfig);
    }

    // 6. Super + M: Maximize/Restore active window
    if modifiers.logo && !modifiers.shift && (sym == keysyms::KEY_m || sym == keysyms::KEY_M) {
        return Some(KeyboardAction::ToggleMaximize);
    }

    // 7. Super + H: Minimize active window
    if modifiers.logo && !modifiers.shift && (sym == keysyms::KEY_h || sym == keysyms::KEY_H) {
        return Some(KeyboardAction::Minimize);
    }

    // 8. Super + Enter: Toggle fullscreen on the active window
    if modifiers.logo && !modifiers.shift && sym == keysyms::KEY_Return {
        return Some(KeyboardAction::ToggleFullscreen);
    }

    // 9. Super + Tab: Cycle window focus
    if modifiers.logo
        && !modifiers.alt
        && (sym == keysyms::KEY_Tab || sym == keysyms::KEY_ISO_Left_Tab)
    {
        return Some(KeyboardAction::CycleWindowFocus {
            forward: !modifiers.shift,
        });
    }

    None
}

/// Executes a compositor action triggered by a shortcut or switcher event.
///
/// # Arguments
///
/// * `state` - Mutable reference to [`BabydraState`].
/// * `action` - The [`KeyboardAction`] to perform.
/// * `serial` - Serial identifier associated with the keyboard event.
pub(crate) fn execute_action(state: &mut BabydraState, action: KeyboardAction, serial: Serial) {
    match action {
        KeyboardAction::QuitCompositor => {
            info!("Quit shortcut pressed: Stopping BabyDraDE compositor...");
            state.loop_signal.stop();
        }
        KeyboardAction::CloseActiveWindow => {
            if let Some(active_window) = state.active_window() {
                info!("Close shortcut pressed: Closing active window with shrink & fade animation");
                state.close_window(&active_window);
            }
        }
        KeyboardAction::LockScreen => {
            info!("Super+L pressed: locking the session");
            lock::spawn();
        }
        KeyboardAction::ReloadConfig => {
            crate::keymap::reload();
            state.reload_config();
        }
        KeyboardAction::ClipboardPicker => {
            info!("Super+V pressed: opening clipboard history");
            crate::shell::clipboard::trigger_picker();
        }
        KeyboardAction::ToggleLauncher => {
            info!("Super+Space pressed: toggling app launcher");
            state.launcher.toggle();
        }
        KeyboardAction::Screenshot => {
            info!("Super+Shift+S pressed: taking screenshot");
            crate::shell::screenshot::trigger();
        }
        KeyboardAction::ToggleMaximize => {
            let active = state.space.elements().next_back().cloned();
            if let Some(active_window) = active {
                info!("Super+M pressed: Toggling maximize on active window");
                state.toggle_maximize(&active_window);
            }
        }
        KeyboardAction::Minimize => {
            let active = state.space.elements().next_back().cloned();
            if let Some(active_window) = active {
                info!("Super+H pressed: Minimizing active window");
                state.minimize_window(&active_window);
            }
        }
        KeyboardAction::ToggleFullscreen => {
            let active = state.space.elements().next_back().cloned();
            if let Some(active_window) = active {
                info!("Super+Enter pressed: Toggling fullscreen on active window");
                state.toggle_fullscreen(&active_window);
            }
        }
        KeyboardAction::CycleWindowFocus { forward } => {
            cycle_window_focus(state, forward, serial);
        }
        KeyboardAction::AltTabStep { backwards } => {
            state.cycle_alt_tab(backwards);
        }
        KeyboardAction::AltTabCancel => {
            if let Some(switcher) = state.alt_tab_switcher.as_mut() {
                switcher.begin_close();
            }
        }
        KeyboardAction::AltTabCommit => {
            let selected = state.alt_tab_switcher.as_ref().and_then(|switcher| {
                switcher
                    .entries
                    .get(switcher.selected_index)
                    .map(|entry| entry.window.clone())
            });
            if let Some(window) = selected {
                state.unminimize_window(&window);
                focus_window(state, window, serial);
            }
            if let Some(switcher) = state.alt_tab_switcher.as_mut() {
                switcher.begin_close();
            }
        }
    }
}
