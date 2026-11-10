//! Touchpad axis and scroll wheel event processing.

use smithay::{
    backend::input::{Axis, AxisRelativeDirection, AxisSource, InputBackend, PointerAxisEvent},
    input::pointer::AxisFrame,
};

use crate::compositor::input::pointer::MouseConfig;
use crate::compositor::input::touchpad::TouchpadConfig;
use crate::compositor::state::BabydraState;

/// Resolves whether natural scrolling should be applied to an axis event, and inverts
/// the scroll values if the underlying backend has not already done so.
///
/// If libinput has already enabled natural scrolling in hardware, `raw_dir` will be
/// [`AxisRelativeDirection::Inverted`] and no double-inversion takes place.
/// On backends where hardware natural scroll is unavailable (e.g. nested Winit),
/// this function software-inverts the displacement deltas.
///
/// # Arguments
///
/// * `source` - The input source of the axis event (finger, wheel, continuous, etc.).
/// * `raw_dir` - The relative direction reported by the input backend.
/// * `amount` - Optional continuous scroll amount in pixels.
/// * `amount_v120` - Optional high-resolution discrete wheel scroll amount.
///
/// # Returns
///
/// Returns a tuple `(adjusted_amount, adjusted_amount_v120, effective_direction)`.
pub(crate) fn apply_natural_scroll(
    source: AxisSource,
    raw_dir: AxisRelativeDirection,
    amount: Option<f64>,
    amount_v120: Option<f64>,
) -> (Option<f64>, Option<f64>, AxisRelativeDirection) {
    let natural_scroll = match source {
        AxisSource::Finger => TouchpadConfig::load().natural_scroll,
        _ => MouseConfig::load().natural_scroll,
    };

    // If the hardware / libinput driver already inverted the direction, pass through.
    if raw_dir == AxisRelativeDirection::Inverted {
        return (amount, amount_v120, AxisRelativeDirection::Inverted);
    }

    // On non-libinput backends or when driver didn't invert, software-invert if configured.
    if natural_scroll {
        let adj_amount = amount.map(|a| -a);
        let adj_v120 = amount_v120.map(|v| -v);
        (adj_amount, adj_v120, AxisRelativeDirection::Inverted)
    } else {
        (amount, amount_v120, AxisRelativeDirection::Identical)
    }
}

/// Processes pointer and touchpad scroll axis events (both 2-finger scroll and wheel ticks).
///
/// Converts backend amounts and high-resolution v120 wheel increments into an [`AxisFrame`],
/// applies natural scrolling configuration when appropriate, and delivers it to the focused
/// Wayland pointer surface.
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

    let (horiz_amount, horiz_v120, horiz_dir) = apply_natural_scroll(
        source,
        event.relative_direction(Axis::Horizontal),
        event.amount(Axis::Horizontal),
        event.amount_v120(Axis::Horizontal),
    );

    let (vert_amount, vert_v120, vert_dir) = apply_natural_scroll(
        source,
        event.relative_direction(Axis::Vertical),
        event.amount(Axis::Vertical),
        event.amount_v120(Axis::Vertical),
    );

    let mut frame = AxisFrame::new(event.time_msec())
        .source(source)
        .relative_direction(Axis::Horizontal, horiz_dir)
        .relative_direction(Axis::Vertical, vert_dir);

    // Process horizontal axis
    if let Some(amount) = horiz_amount {
        if source == AxisSource::Finger && amount == 0.0 {
            frame = frame.stop(Axis::Horizontal);
        } else {
            frame = frame.value(Axis::Horizontal, amount);
        }
    } else if let Some(amount_v120) = horiz_v120 {
        frame = frame.value(Axis::Horizontal, amount_v120 / 8.0);
    }
    if let Some(amount_v120) = horiz_v120 {
        frame = frame.v120(Axis::Horizontal, amount_v120.round() as i32);
    }

    // Process vertical axis
    if let Some(amount) = vert_amount {
        if source == AxisSource::Finger && amount == 0.0 {
            frame = frame.stop(Axis::Vertical);
        } else {
            frame = frame.value(Axis::Vertical, amount);
        }
    } else if let Some(amount_v120) = vert_v120 {
        frame = frame.value(Axis::Vertical, amount_v120 / 8.0);
    }
    if let Some(amount_v120) = vert_v120 {
        frame = frame.v120(Axis::Vertical, amount_v120.round() as i32);
    }

    if let Some(pointer) = state.seat.get_pointer() {
        pointer.axis(state, frame);
        pointer.frame(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use smithay::backend::input::AxisRelativeDirection;

    #[test]
    fn test_apply_natural_scroll_passthrough_when_already_inverted() {
        let (amt, v120, dir) = apply_natural_scroll(
            AxisSource::Finger,
            AxisRelativeDirection::Inverted,
            Some(-15.0),
            Some(-120.0),
        );
        assert_eq!(amt, Some(-15.0));
        assert_eq!(v120, Some(-120.0));
        assert_eq!(dir, AxisRelativeDirection::Inverted);
    }

    #[test]
    fn test_apply_natural_scroll_identical_when_disabled() {
        // Defaults: natural_scroll is false
        let (amt, v120, dir) = apply_natural_scroll(
            AxisSource::Wheel,
            AxisRelativeDirection::Identical,
            Some(15.0),
            Some(120.0),
        );
        assert_eq!(amt, Some(15.0));
        assert_eq!(v120, Some(120.0));
        assert_eq!(dir, AxisRelativeDirection::Identical);
    }
}
