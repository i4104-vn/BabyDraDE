use smithay::{
    backend::input::{Event, InputBackend, KeyState, KeyboardKeyEvent},
    input::keyboard::{keysyms, FilterResult},
    utils::SERIAL_COUNTER,
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
    let mut cycle_focus = false;
    let mut quit_compositor = false;
    let mut close_active_window = false;

    keyboard.input::<(), _>(
        state,
        event.key_code(),
        event.state(),
        serial,
        time,
        |_state, modifiers, handle| {
            if event.state() == KeyState::Pressed {
                let sym = handle.modified_sym();

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

                // 3. Super (Win) + T: Spawn new terminal window
                if modifiers.logo
                    && (sym == keysyms::KEY_t.into() || sym == keysyms::KEY_T.into())
                {
                    spawn_terminal = true;
                    return FilterResult::Intercept(());
                }

                // 4. Alt + Tab: Cycle windows and raise next to top
                if modifiers.alt
                    && (sym == keysyms::KEY_Tab.into() || sym == keysyms::KEY_ISO_Left_Tab.into())
                {
                    cycle_focus = true;
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

    if cycle_focus {
        let next_window = state.space.elements().rev().nth(1).cloned();
        if let Some(next_window) = next_window {
            state.space.raise_element(&next_window, true);
            keyboard.set_focus(
                state,
                Some(next_window.toplevel().unwrap().wl_surface().clone()),
                serial,
            );
            state.space.elements().for_each(|window| {
                window.toplevel().unwrap().send_pending_configure();
            });
            state.update_active_window(Some(&next_window));
        }
    }
}
