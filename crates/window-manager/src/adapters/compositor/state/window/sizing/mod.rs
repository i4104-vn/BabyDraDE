use crate::application::windows::{default_position, maximize_target, restore_target};
use crate::domain::geometry::{Point as PurePoint, Rectangle as PureRectangle, Size as PureSize};
use smithay::{
    desktop::Window,
    reexports::{
        wayland_protocols::xdg::shell::server::xdg_toplevel,
        wayland_server::protocol::{wl_output::WlOutput, wl_surface::WlSurface},
    },
    utils::{Logical, Point, Rectangle, Size, SERIAL_COUNTER},
};

use super::state_data::lock_window_state;
use crate::adapters::compositor::state::{
    cancel_minimizing_animation, is_window_exiting, start_window_minimizing,
    start_window_restoring, start_window_transition, BabydraState,
};

// Smithay geometry is converted to and from pure domain values only here, at
// the boundary between the protocol adapter and the layout use cases.

fn to_pure_point(point: Point<i32, Logical>) -> PurePoint<i32> {
    PurePoint::from((point.x, point.y))
}

fn to_pure_size(size: Size<i32, Logical>) -> PureSize<i32> {
    PureSize::from((size.w, size.h))
}

fn to_pure_rect(rect: Rectangle<i32, Logical>) -> PureRectangle<i32> {
    PureRectangle::new(to_pure_point(rect.loc), to_pure_size(rect.size))
}

fn to_smithay_point(point: PurePoint<i32>) -> Point<i32, Logical> {
    Point::from((point.x, point.y))
}

fn to_smithay_size(size: PureSize<i32>) -> Size<i32, Logical> {
    Size::from((size.w, size.h))
}

fn to_smithay_rect(rect: PureRectangle<i32>) -> Rectangle<i32, Logical> {
    Rectangle::new(to_smithay_point(rect.loc), to_smithay_size(rect.size))
}

fn default_smithay_position() -> Point<i32, Logical> {
    to_smithay_point(default_position())
}

fn smithay_restore_target(
    saved: Option<(Point<i32, Logical>, Size<i32, Logical>)>,
    current_size: Size<i32, Logical>,
) -> (Point<i32, Logical>, Size<i32, Logical>) {
    let (loc, size) = restore_target(
        saved.map(|(loc, size)| (to_pure_point(loc), to_pure_size(size))),
        to_pure_size(current_size),
    );
    (to_smithay_point(loc), to_smithay_size(size))
}

mod sizing_maximize;
mod sizing_minimize;
mod sizing_fullscreen;
