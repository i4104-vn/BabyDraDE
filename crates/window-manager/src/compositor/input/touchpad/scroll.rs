//! Touchpad axis and scroll wheel event processing.

use smithay::{
    backend::input::{Axis, AxisSource, InputBackend, PointerAxisEvent},
    input::pointer::AxisFrame,
};

use crate::compositor::state::BabydraState;

/// Processes pointer and touchpad scroll axis events (both 2-finger scroll and wheel ticks).
///
/// Converts backend amounts and high-resolution v120 wheel increments into an [`AxisFrame`]
/// and delivers it to the focused Wayland pointer surface.
///
/// # Arguments
///
/// * `state` - Mutable reference to [`BabydraState`].
/// * `event` - The pointer axis event providing vertical and horizontal scroll amounts.
pub(crate) fn handle_axis<B, E>(state: &mut BabydraState, event: E)
where
    B: InputBackend,
    E: PointerAxisEvent<B>,
{
    state.cursor_visible = true;
    let source = event.source();
    let mut frame = AxisFrame::new(event.time_msec())
        .source(source)
        .relative_direction(Axis::Horizontal, event.relative_direction(Axis::Horizontal))
        .relative_direction(Axis::Vertical, event.relative_direction(Axis::Vertical));

    for axis in [Axis::Horizontal, Axis::Vertical] {
        let amount = event.amount(axis);
        let amount_v120 = event.amount_v120(axis);

        if let Some(amount) = amount {
            if source == AxisSource::Finger && amount == 0.0 {
                frame = frame.stop(axis);
            } else {
                frame = frame.value(axis, amount);
            }
        } else if let Some(amount_v120) = amount_v120 {
            // Provide a legacy pixel value as well as v120 for wheel-only backends.
            frame = frame.value(axis, amount_v120 / 8.0);
        }

        if let Some(amount_v120) = amount_v120 {
            frame = frame.v120(axis, amount_v120.round() as i32);
        }
    }

    if let Some(pointer) = state.seat.get_pointer() {
        pointer.axis(state, frame);
        pointer.frame(state);
    }
}

