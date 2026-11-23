//! Touch interaction with the launcher and switcher overlays.

use smithay::{
    backend::input::TouchSlot,
    utils::{Logical, Point, SERIAL_COUNTER},
};

use crate::adapters::input::{InputContext, OverlayTouchState};

/// Checks whether a touch-down occurs on an active overlay (launcher or switcher)
/// and registers tracking state instead of immediately launching/activating.
pub(super) fn handle_overlay_touch_down<C: InputContext>(
    state: &mut C,
    slot: TouchSlot,
    position: Point<f64, Logical>,
) -> bool {
    if !state.switcher_is_active() && state.launcher_is_visible() {
        *state.overlay_touch_mut() = Some(OverlayTouchState {
            slot,
            start_pos: position,
            last_pos: position,
            has_dragged: false,
            is_switcher: false,
        });
        return true;
    }

    if state.switcher_is_active() {
        *state.overlay_touch_mut() = Some(OverlayTouchState {
            slot,
            start_pos: position,
            last_pos: position,
            has_dragged: false,
            is_switcher: true,
        });
        return true;
    }

    false
}

/// Routes finger movement to an active overlay drag or hover.
///
/// Returns `true` when the event was consumed by an overlay.
pub(super) fn handle_overlay_motion<C: InputContext>(
    state: &mut C,
    slot: TouchSlot,
    position: Point<f64, Logical>,
) -> bool {
    if let Some(mut touch) = state.overlay_touch_mut().take() {
        if touch.slot == slot {
            let dy = position.y - touch.last_pos.y;
            let total_dy = (position.y - touch.start_pos.y).abs();
            touch.last_pos = position;

            if total_dy > 8.0 || touch.has_dragged {
                touch.has_dragged = true;
                let slots = -dy / common::constants::launcher::SLOT;
                if touch.is_switcher {
                    state.switcher_drag_by(slots);
                } else if state.launcher_is_visible() {
                    state.launcher_drag_by(slots);
                }
            } else if !touch.is_switcher && state.launcher_is_visible() {
                state.launcher_update_hover(position);
            }

            *state.overlay_touch_mut() = Some(touch);
            return true;
        } else {
            *state.overlay_touch_mut() = Some(touch);
        }
    }

    false
}

/// Finishes an overlay drag or handles an overlay tap on touch-up.
///
/// Returns `true` when the event was consumed by an overlay.
pub(super) fn handle_overlay_up<C: InputContext>(state: &mut C, slot: TouchSlot) -> bool {
    if let Some(touch) = state.overlay_touch_mut().take() {
        if touch.slot == slot {
            if touch.has_dragged {
                // Drag completed: snap to nearest slot
                if touch.is_switcher {
                    state.switcher_finish_drag();
                } else if state.launcher_is_visible() {
                    state.launcher_finish_drag();
                }
            } else {
                // Tap gesture: launch app or select window
                if touch.is_switcher {
                    let serial = SERIAL_COUNTER.next_serial();
                    let slide = state.switcher_slide(common::constants::launcher::PANEL_WIDTH);
                    let tapped = state
                        .switcher_index_at((touch.start_pos.x, touch.start_pos.y), slide)
                        .and_then(|index| state.switcher_window_at(index));

                    if let Some(window) = tapped {
                        state.unminimize_window(&window);
                        crate::adapters::input::keyboard::focus::focus_window(
                            state, window, serial,
                        );
                    }
                    state.switcher_begin_close();
                } else if state.launcher_is_visible() {
                    if let Some(index) = state.launcher_index_at(touch.start_pos) {
                        state.launcher_launch(index);
                    } else {
                        state.launcher_close();
                    }
                }
            }
            return true;
        } else {
            *state.overlay_touch_mut() = Some(touch);
        }
    }

    false
}

/// Finishes an overlay drag when the contact is cancelled.
///
/// Returns `true` when the event was consumed by an overlay.
pub(super) fn handle_overlay_cancel<C: InputContext>(state: &mut C, slot: TouchSlot) -> bool {
    if let Some(touch) = state.overlay_touch_mut().take() {
        if touch.slot == slot {
            if touch.has_dragged {
                if touch.is_switcher {
                    state.switcher_finish_drag();
                } else if state.launcher_is_visible() {
                    state.launcher_finish_drag();
                }
            }
            return true;
        } else {
            *state.overlay_touch_mut() = Some(touch);
        }
    }

    false
}
