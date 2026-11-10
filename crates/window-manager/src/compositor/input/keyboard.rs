use smithay::{
    backend::input::{Event, InputBackend, KeyState, KeyboardKeyEvent},
    input::keyboard::{keysyms, FilterResult},
    utils::{Serial, SERIAL_COUNTER},
};
use tracing::info;

use super::super::state::BabydraState;
use super::launcher::spawn_new_terminal;

pub(crate) fn handle_keyboard<B: InputBackend, K: KeyboardKeyEvent<B>>(
    state: &mut BabydraState,
    event: K,
) {
    let serial = SERIAL_COUNTER.next_serial();
    let time = Event::time_msec(&event);
    let keyboard = state.seat.get_keyboard().unwrap();

    let mut spawn_terminal = false;
    let mut switcher_step = None;
    let mut cancel_switcher = false;
    let mut commit_switcher = false;
    let mut quit_compositor = false;
    let mut close_active_window = false;
    let mut toggle_blur = false;
    let mut toggle_maximize_active = false;
    let mut minimize_active = false;
    let mut toggle_fullscreen_active = false;
    let mut cycle_window_step = None;

    keyboard.input::<(), _>(
        state,
        event.key_code(),
        event.state(),
        serial,
        time,
        |state, modifiers, handle| {
            let sym = handle.modified_sym();
            let switcher_active = state.alt_tab_switcher.is_some();

            if switcher_active
                && event.state() == KeyState::Released
                && (sym == keysyms::KEY_Tab.into() || sym == keysyms::KEY_ISO_Left_Tab.into())
            {
                return FilterResult::Intercept(());
            }

            if switcher_active
                && event.state() == KeyState::Pressed
                && sym == keysyms::KEY_Escape.into()
                && !modifiers.logo
            {
                cancel_switcher = true;
                return FilterResult::Intercept(());
            }

            if switcher_active
                && event.state() == KeyState::Released
                && (!modifiers.alt
                    || sym == keysyms::KEY_Alt_L.into()
                    || sym == keysyms::KEY_Alt_R.into())
            {
                commit_switcher = true;
            }

            if event.state() == KeyState::Pressed {
                // 1. Super + Escape OR Super + Shift + Q OR Ctrl + Alt + Backspace: Quit WM immediately
                if (modifiers.logo && sym == keysyms::KEY_Escape.into())
                    || (modifiers.logo
                        && modifiers.shift
                        && (sym == keysyms::KEY_q.into() || sym == keysyms::KEY_Q.into()))
                    || (modifiers.ctrl && modifiers.alt && sym == keysyms::KEY_BackSpace.into())
                {
                    quit_compositor = true;
                    return FilterResult::Intercept(());
                }

                // 2. Super + Q: Close active window
                if modifiers.logo
                    && !modifiers.shift
                    && (sym == keysyms::KEY_q.into() || sym == keysyms::KEY_Q.into())
                {
                    close_active_window = true;
                    return FilterResult::Intercept(());
                }

                // 3. Super (Win) + T OR Super + Return: Spawn new terminal window
                if modifiers.logo
                    && (sym == keysyms::KEY_t.into()
                        || sym == keysyms::KEY_T.into()
                        || sym == keysyms::KEY_Return.into())
                {
                    spawn_terminal = true;
                    return FilterResult::Intercept(());
                }

                // 4. Alt + Tab: choose a window, then focus it when Alt is released
                if modifiers.alt
                    && (sym == keysyms::KEY_Tab.into() || sym == keysyms::KEY_ISO_Left_Tab.into())
                {
                    switcher_step =
                        Some(modifiers.shift || sym == keysyms::KEY_ISO_Left_Tab.into());
                    return FilterResult::Intercept(());
                }

                // 5. Super + B: Toggle blur on the active window
                if modifiers.logo
                    && (sym == keysyms::KEY_b.into() || sym == keysyms::KEY_B.into())
                {
                    toggle_blur = true;
                    return FilterResult::Intercept(());
                }

                // 6. Super + M: Maximize/Restore active window
                if modifiers.logo
                    && !modifiers.shift
                    && (sym == keysyms::KEY_m.into() || sym == keysyms::KEY_M.into())
                {
                    toggle_maximize_active = true;
                    return FilterResult::Intercept(());
                }

                // 7. Super + H: Minimize active window
                if modifiers.logo
                    && !modifiers.shift
                    && (sym == keysyms::KEY_h.into() || sym == keysyms::KEY_H.into())
                {
                    minimize_active = true;
                    return FilterResult::Intercept(());
                }

                // 8. Super + F: Toggle fullscreen on the active window
                if modifiers.logo
                    && !modifiers.shift
                    && (sym == keysyms::KEY_f.into() || sym == keysyms::KEY_F.into())
                {
                    toggle_fullscreen_active = true;
                    return FilterResult::Intercept(());
                }

                // 9. Super + Tab: Cycle window focus
                if modifiers.logo
                    && !modifiers.alt
                    && (sym == keysyms::KEY_Tab.into() || sym == keysyms::KEY_ISO_Left_Tab.into())
                {
                    cycle_window_step = Some(!modifiers.shift);
                    return FilterResult::Intercept(());
                }
            }
            FilterResult::Forward
        },
    );

    if quit_compositor {
        info!("Quit shortcut pressed: Stopping BabyDraDE compositor...");
        state.loop_signal.stop();
        return;
    }

    if close_active_window {
        if let Some(active_window) = state.space.elements().next_back() {
            info!("Super+Q pressed: Closing active window");
            active_window.toplevel().unwrap().send_close();
            let next_top = state.space.elements().rev().nth(1).cloned();
            state.update_active_window(next_top.as_ref());
        }
    }

    if spawn_terminal {
        spawn_new_terminal(&state.socket_name);
    }

    if toggle_blur {
        if let Some((app_id, is_now_blurred)) = state.toggle_blur_active_window() {
            info!(
                "Super+B pressed: Toggled blur on active app '{}': blur_enabled={}",
                app_id, is_now_blurred
            );
        } else {
            info!("Super+B pressed: No active window found to toggle blur on");
        }
    }

    if toggle_maximize_active {
        let active = state.space.elements().next_back().cloned();
        if let Some(active_window) = active {
            info!("Super+M pressed: Toggling maximize on active window");
            state.toggle_maximize(&active_window);
        }
    }

    if minimize_active {
        let active = state.space.elements().next_back().cloned();
        if let Some(active_window) = active {
            info!("Super+H pressed: Minimizing active window");
            state.minimize_window(&active_window);
        }
    }

    if toggle_fullscreen_active {
        let active = state.space.elements().next_back().cloned();
        if let Some(active_window) = active {
            info!("Super+F pressed: Toggling fullscreen on active window");
            state.toggle_fullscreen(&active_window);
        }
    }

    if let Some(forward) = cycle_window_step {
        cycle_window_focus(state, forward, serial);
    }

    if cancel_switcher {
        state.alt_tab_switcher = None;
    } else if let Some(backwards) = switcher_step {
        state.cycle_alt_tab(backwards);
    } else if commit_switcher {
        if let Some(switcher) = state.alt_tab_switcher.take() {
            if let Some(window) = switcher
                .entries
                .get(switcher.selected_index)
                .map(|entry| &entry.window)
            {
                state.unminimize_window(window);
                focus_window(state, window.clone(), serial);
            }
        }
    }
}

pub(crate) fn cycle_window_focus(state: &mut BabydraState, forward: bool, serial: Serial) -> bool {
    state.cleanup_windows();
    if state.windows.is_empty() {
        return false;
    }

    let next_window = if state.windows.len() == 1 {
        state.windows.first().cloned()
    } else if forward {
        state.windows.iter().rev().nth(1).cloned()
    } else {
        state.windows.first().cloned()
    };

    let Some(next_window) = next_window else {
        return false;
    };

    state.unminimize_window(&next_window);
    focus_window(state, next_window, serial);
    true
}

fn focus_window(state: &mut BabydraState, window: smithay::desktop::Window, serial: Serial) {
    let Some(toplevel) = window.toplevel() else {
        return;
    };
    let Some(keyboard) = state.seat.get_keyboard() else {
        return;
    };

    state.space.raise_element(&window, true);
    state.raise_window(&window);
    keyboard.set_focus(state, Some(toplevel.wl_surface().clone()), serial);
    state.space.elements().for_each(|w| {
        let active = w == &window;
        w.set_activated(active);
        if let Some(toplevel) = w.toplevel() {
            toplevel.send_pending_configure();
        }
    });
    state.update_active_window(Some(&window));
}
