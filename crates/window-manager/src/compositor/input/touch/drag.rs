//! Touchscreen interactive window movement and resizing handling.

use smithay::utils::Point;

use crate::compositor::state::BabydraState;

/// Processes active interactive window movement driven by touchscreen dragging.
///
/// Ensures a minimum distance threshold (16 square pixels) before beginning window translation.
pub(crate) fn handle_touch_drag(
    state: &mut BabydraState,
    position: Point<f64, smithay::utils::Logical>,
) -> bool {
    if let Some(mut move_state) = state.move_state.take() {
        let move_delta = position - move_state.initial_pointer_location;
        let dist_sq = move_delta.x * move_delta.x + move_delta.y * move_delta.y;
        if !move_state.has_dragged && dist_sq < 16.0 {
            state.move_state = Some(move_state);
            return true;
        }

        if !move_state.has_dragged {
            let is_maximized = move_state
                .window
                .toplevel()
                .map(|top| {
                    top.current_state().states.contains(
                        smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::State::Maximized,
                    )
                })
                .unwrap_or(false);
            if is_maximized {
                let window = move_state.window.clone();
                let saved_size = crate::compositor::state::window::state_data::window_state(&window)
                    .lock()
                    .unwrap()
                    .saved_size;
                state.unmaximize_window(&window);
                let size = saved_size.unwrap_or_else(|| window.geometry().size);
                let desired = Point::from((
                    position.x.round() as i32 - size.w / 2,
                    position.y.round() as i32 + crate::rendering::titlebar::HEIGHT / 2,
                ));
                move_state.initial_window_location = desired
                    - Point::from((move_delta.x.round() as i32, move_delta.y.round() as i32));
            }
        }

        move_state.has_dragged = true;
        state.last_titlebar_click = None;

        let new_loc = move_state.initial_window_location
            + Point::from((move_delta.x.round() as i32, move_delta.y.round() as i32));
        state
            .space
            .map_element(move_state.window.clone(), new_loc, false);
        state.move_state = Some(move_state);
        return true;
    }
    false
}

/// Processes active interactive window resizing driven by touchscreen dragging.
pub(crate) fn handle_touch_resize(
    state: &mut BabydraState,
    position: Point<f64, smithay::utils::Logical>,
) -> bool {
    if let Some(resize_state) = state.resize_state.take() {
        let dx = (position.x - resize_state.initial_pointer_location.x).round() as i32;
        let dy = (position.y - resize_state.initial_pointer_location.y).round() as i32;

        let init_loc = resize_state.initial_window_location;
        let init_size = resize_state.initial_window_size;

        let (min_w, min_h, max_w, max_h) = (150, 100, i32::MAX, i32::MAX);

        let mut new_x = init_loc.x;
        let mut new_y = init_loc.y;
        let mut new_w = init_size.w;
        let mut new_h = init_size.h;

        if resize_state.resizes_left() {
            let target_w = (init_size.w - dx).clamp(min_w, max_w);
            new_w = target_w;
            new_x = init_loc.x + (init_size.w - target_w);
        } else if resize_state.resizes_right() {
            new_w = (init_size.w + dx).clamp(min_w, max_w);
        }

        if resize_state.resizes_top() {
            let target_h = (init_size.h - dy).clamp(min_h, max_h);
            new_h = target_h;
            new_y = init_loc.y + (init_size.h - target_h);
        } else if resize_state.resizes_bottom() {
            new_h = (init_size.h + dy).clamp(min_h, max_h);
        }

        if let Some(toplevel) = resize_state.window.toplevel() {
            toplevel.with_pending_state(|state| {
                state.states.set(
                    smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::State::Resizing,
                );
                state.size = Some((new_w, new_h).into());
            });
            toplevel.send_configure();
        }

        if new_x != init_loc.x || new_y != init_loc.y {
            state
                .space
                .map_element(resize_state.window.clone(), (new_x, new_y), false);
        }

        state.resize_state = Some(resize_state);
        return true;
    }
    false
}
