//! Per-slot touch contact bookkeeping and gesture tracking entry points.

use smithay::{
    backend::input::TouchSlot,
    utils::{Logical, Point},
};

use crate::adapters::input::touch::gestures::{on_touch_down, on_touch_motion, on_touch_up};
use crate::adapters::input::InputContext;

/// Registers a newly contacted finger for gesture tracking.
pub(super) fn track_down<C: InputContext>(
    state: &mut C,
    slot: TouchSlot,
    position: Point<f64, Logical>,
    time_ms: u64,
) {
    on_touch_down(state, slot, position, time_ms);
}

/// Updates the gesture position of an already tracked finger.
pub(super) fn track_motion<C: InputContext>(
    state: &mut C,
    slot: TouchSlot,
    position: Point<f64, Logical>,
) {
    on_touch_motion(state, slot, position);
}

/// Evaluates and clears the gesture data of a lifting finger.
pub(super) fn track_up<C: InputContext>(state: &mut C, slot: TouchSlot, time_ms: u64) {
    on_touch_up(state, slot, time_ms);
}

/// Records a new touch slot.
pub(super) fn begin_contact<C: InputContext>(state: &mut C, slot: TouchSlot) {
    state.touch_slots_mut().insert(slot);
}

/// Removes a released or cancelled touch slot.
pub(super) fn end_contact<C: InputContext>(state: &mut C, slot: TouchSlot) {
    state.touch_slots_mut().remove(&slot);
}

/// Reports whether all fingers have left the surface.
pub(super) fn no_contacts<C: InputContext>(state: &C) -> bool {
    state.touch_slots().is_empty()
}

/// Finalizes pending interactive move/resize state when the last finger lifts.
pub(super) fn finish_interactions<C: InputContext>(state: &mut C) {
    if let Some(move_state) = state.move_state_mut().take() {
        if move_state.has_dragged {
            if let Some(toplevel) = move_state.window.toplevel() {
                toplevel.send_configure();
            }
        }
    }
    if let Some(resize_state) = state.resize_state_mut().take() {
        if let Some(toplevel) = resize_state.window.toplevel() {
            toplevel.with_pending_state(|state| {
                state.states.unset(
                    smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::State::Resizing,
                );
            });
            toplevel.send_configure();
        }
    }
}
