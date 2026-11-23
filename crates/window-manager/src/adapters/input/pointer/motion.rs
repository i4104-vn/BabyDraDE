//! Pointer motion handling for relative and absolute pointing devices.

use smithay::{
    backend::input::{AbsolutePositionEvent, InputBackend, PointerMotionEvent},
    input::pointer::MotionEvent,
    utils::{Logical, Point, SERIAL_COUNTER},
};

use super::drag::{handle_window_drag, handle_window_resize};
use crate::adapters::compositor::state::BabydraState;
use crate::adapters::input::common::transform_absolute_position;

fn update_overlay_hover(state: &mut BabydraState, position: Point<f64, Logical>) {
    if state.alt_tab_switcher.is_none() {
        state.launcher.update_hover(position);
        return;
    }

    let output = state.space.outputs().next().cloned();
    let (size, scale) = output
        .as_ref()
        .and_then(|output| {
            let mode = output.current_mode()?;
            let size = output.current_transform().transform_size(mode.size);
            Some((
                size,
                smithay::utils::Scale::from(output.current_scale().fractional_scale()),
            ))
        })
        .unwrap_or(((1920, 1080).into(), smithay::utils::Scale::from(1.0)));
    let logical_w = (size.w as f64 / scale.x).max(1.0);
    let logical_h = (size.h as f64 / scale.y).max(1.0);

    let index = state.alt_tab_switcher.as_ref().and_then(|switcher| {
        let layout =
            crate::adapters::shell::switcher::view::compute_layout(switcher, logical_w, logical_h)?;
        let slide =
            (1.0 - switcher.presence().clamp(0.0, 1.0)) * common::constants::launcher::PANEL_WIDTH;
        layout.panel.index_at(
            (position.x, position.y),
            slide,
            switcher.entries.len(),
            switcher.visual_selection(),
        )
    });

    if let Some(switcher) = state.alt_tab_switcher.as_mut() {
        switcher.set_hover(index);
    }
}

/// Handles relative pointer motion events (such as mice and touchpad cursor movements).
///
/// Updates the compositor pointer location clamped to screen boundaries, drives active
/// window drags, updates pointer focus, and delivers motion frames to client surfaces.
///
/// # Arguments
///
/// * `state` - Mutable reference to [`BabydraState`].
/// * `event` - The relative pointer motion event containing delta x and y.
pub(crate) fn handle_pointer_motion<B: InputBackend, P: PointerMotionEvent<B>>(
    state: &mut BabydraState,
    event: P,
) {
    state.cursor_visible = true;
    let pointer = state.seat.get_pointer().unwrap();
    let delta = event.delta();
    let mut position = state.pointer_location + delta;

    if let Some(output) = state.space.outputs().next() {
        if let Some(geo) = state.space.output_geometry(output) {
            let max_x = (geo.loc.x + geo.size.w) as f64;
            let max_y = (geo.loc.y + geo.size.h) as f64;
            position.x = position.x.clamp(geo.loc.x as f64, max_x);
            position.y = position.y.clamp(geo.loc.y as f64, max_y);
        }
    }

    state.pointer_location = position;

    update_overlay_hover(state, position);

    if handle_window_resize(state, position) {
        return;
    }

    if handle_window_drag(state, position, event.time_msec()) {
        return;
    }

    let under = state.surface_under(position);
    pointer.motion(
        state,
        under,
        &MotionEvent {
            location: position,
            serial: SERIAL_COUNTER.next_serial(),
            time: event.time_msec(),
        },
    );
    pointer.frame(state);
}

/// Handles absolute pointer motion events (such as drawing tablets or virtual VM pointers).
///
/// Converts hardware absolute coordinates to logical coordinates using output geometry,
/// clamps to screen boundaries, updates window drags, and forwards motion to Wayland clients.
///
/// # Arguments
///
/// * `state` - Mutable reference to [`BabydraState`].
/// * `event` - The absolute position event providing coordinates.
pub(crate) fn handle_pointer_motion_absolute<B: InputBackend, P: AbsolutePositionEvent<B>>(
    state: &mut BabydraState,
    event: P,
) {
    state.cursor_visible = true;
    let Some(position) = transform_absolute_position(state, &event, false) else {
        return;
    };
    let pointer = state.seat.get_pointer().unwrap();

    state.pointer_location = position;

    update_overlay_hover(state, position);

    if handle_window_resize(state, position) {
        return;
    }

    if handle_window_drag(state, position, event.time_msec()) {
        return;
    }

    let under = state.surface_under(position);
    pointer.motion(
        state,
        under,
        &MotionEvent {
            location: position,
            serial: SERIAL_COUNTER.next_serial(),
            time: event.time_msec(),
        },
    );
    pointer.frame(state);
}
