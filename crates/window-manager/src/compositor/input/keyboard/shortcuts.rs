//! Global keyboard shortcut definitions, matching, and action execution.

use smithay::input::keyboard::{keysyms, ModifiersState};
use smithay::utils::Serial;
use tracing::info;

use super::focus::{cycle_window_focus, focus_window};
use super::launcher::spawn_new_terminal;
use crate::compositor::state::BabydraState;
use crate::rendering::lock;

/// High-level compositor actions triggered by keyboard shortcuts.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum KeyboardAction {
    /// Gracefully shut down the compositor loop.
    QuitCompositor,
    /// Send a close request to the top/active window.
    CloseActiveWindow,
    /// Spawn a new terminal emulator instance.
    SpawnTerminal,
    /// Show the integrated lock screen.
    LockScreen,
    /// Toggle background blur shader on the active window.
    ToggleBlur,
    /// Toggle maximize state on the active window.
    ToggleMaximize,
    /// Minimize the active window.
    Minimize,
    /// Toggle fullscreen state on the active window.
    ToggleFullscreen,
    /// Cycle focus between open windows.
    CycleWindowFocus { forward: bool },
    /// Step through the Alt+Tab window switcher overlay.
    AltTabStep { backwards: bool },
    /// Cancel the Alt+Tab switcher without changing focus.
    AltTabCancel,
    /// Commit the Alt+Tab switcher selection and focus chosen window.
    AltTabCommit,
}

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

    // 2. Super + Q: Close active window
    if modifiers.logo && !modifiers.shift && (sym == keysyms::KEY_q || sym == keysyms::KEY_Q) {
        return Some(KeyboardAction::CloseActiveWindow);
    }

    // 3. Super (Win) + T OR Super + Return: Spawn new terminal window
    if modifiers.logo
        && (sym == keysyms::KEY_t || sym == keysyms::KEY_T || sym == keysyms::KEY_Return)
    {
        return Some(KeyboardAction::SpawnTerminal);
    }

    // 4. Super + L: Lock the current session
    if modifiers.logo && !modifiers.shift && (sym == keysyms::KEY_l || sym == keysyms::KEY_L) {
        return Some(KeyboardAction::LockScreen);
    }

    // 5. Alt + Tab: choose a window, then focus it when Alt is released
    if modifiers.alt && (sym == keysyms::KEY_Tab || sym == keysyms::KEY_ISO_Left_Tab) {
        let backwards = modifiers.shift || sym == keysyms::KEY_ISO_Left_Tab;
        return Some(KeyboardAction::AltTabStep { backwards });
    }

    // 5. Super + B: Toggle blur on the active window
    if modifiers.logo && (sym == keysyms::KEY_b || sym == keysyms::KEY_B) {
        return Some(KeyboardAction::ToggleBlur);
    }

    // 6. Super + M: Maximize/Restore active window
    if modifiers.logo && !modifiers.shift && (sym == keysyms::KEY_m || sym == keysyms::KEY_M) {
        return Some(KeyboardAction::ToggleMaximize);
    }

    // 7. Super + H: Minimize active window
    if modifiers.logo && !modifiers.shift && (sym == keysyms::KEY_h || sym == keysyms::KEY_H) {
        return Some(KeyboardAction::Minimize);
    }

    // 8. Super + F: Toggle fullscreen on the active window
    if modifiers.logo && !modifiers.shift && (sym == keysyms::KEY_f || sym == keysyms::KEY_F) {
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
            if let Some(active_window) = state.space.elements().next_back() {
                info!("Super+Q pressed: Closing active window");
                active_window.toplevel().unwrap().send_close();
                let next_top = state.space.elements().rev().nth(1).cloned();
                state.update_active_window(next_top.as_ref());
            }
        }
        KeyboardAction::SpawnTerminal => {
            spawn_new_terminal(&state.socket_name);
        }
        KeyboardAction::LockScreen => {
            info!("Super+L pressed: locking the session");
            lock::spawn();
        }
        KeyboardAction::ToggleBlur => {
            if let Some((app_id, is_now_blurred)) = state.toggle_blur_active_window() {
                info!(
                    "Super+B pressed: Toggled blur on active app '{}': blur_enabled={}",
                    app_id, is_now_blurred
                );
            } else {
                info!("Super+B pressed: No active window found to toggle blur on");
            }
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
                info!("Super+F pressed: Toggling fullscreen on active window");
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
            if let Some(switcher) = state.alt_tab_switcher.take() {
                state.restore_alt_tab_windows(&switcher);
            }
        }
        KeyboardAction::AltTabCommit => {
            if let Some(switcher) = state.alt_tab_switcher.take() {
                state.restore_alt_tab_windows(&switcher);
                if let Some(entry) = switcher.entries.get(switcher.selected_index) {
                    state.unminimize_window(&entry.window);
                    focus_window(state, entry.window.clone(), serial);
                }
            }
        }
    }
}
