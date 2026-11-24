//! Keyboard input processing, focus handling, and global hotkeys.
//!
//! Facade that assembles the per-family key handlers around the seat filter.

use smithay::{
    backend::input::{Event, InputBackend, KeyboardKeyEvent},
    input::keyboard::FilterResult,
    utils::SERIAL_COUNTER,
};

use crate::adapters::input::InputContext;

use super::shortcuts::execute_action;

#[path = "key_launcher.rs"]
mod launcher;
#[path = "key_models.rs"]
mod models;
#[path = "key_shortcuts.rs"]
mod shortcuts;
#[path = "key_switcher.rs"]
mod switcher;

use models::KeyContext;

/// Dispatches and processes an incoming keyboard event from the input backend.
///
/// Filters global window manager shortcuts (such as Super+Q to close window,
/// Alt+Tab window switching, and Super+Enter for fullscreen), and forwards non-intercepted
/// key events to the Wayland surface holding keyboard focus.
///
/// # Arguments
/// * `state` - Mutable input context driving the compositor.
/// * `event` - Backend keyboard event to process.
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
            let switcher_active = state.switcher_is_active();
            let mut ctx = KeyContext {
                state,
                key_state: event.state(),
                modifiers,
                sym: handle.modified_sym().into(),
                action: &mut action_to_execute,
                alt_tab_hold: &mut alt_tab_hold,
                switcher_active,
            };

            if launcher::handle(&mut ctx, &handle) {
                return FilterResult::Intercept(());
            }
            if switcher::handle(&mut ctx) {
                return FilterResult::Intercept(());
            }
            if shortcuts::handle(&mut ctx, &handle) {
                return FilterResult::Intercept(());
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
