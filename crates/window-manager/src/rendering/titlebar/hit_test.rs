//! Titlebar pointer hit testing.

use smithay::{
    desktop::Window,
    utils::{Logical, Point},
};

use crate::enums::{
    Action, CORNER_RADIUS, TITLEBAR_BUTTON_RADIUS as BUTTON, TITLEBAR_HEIGHT as HEIGHT,
};

pub fn hit_test(
    window: &Window,
    location: Point<i32, Logical>,
    point: Point<f64, Logical>,
) -> Option<Action> {
    let state = window.toplevel()?.current_state();
    if state.decoration_mode != Some(
        smithay::reexports::wayland_protocols::xdg::decoration::zv1::server::zxdg_toplevel_decoration_v1::Mode::ServerSide,
    ) || state.states.contains(
        smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::State::Fullscreen,
    ) {
        return None;
    }
    let maximized = state.states.contains(
        smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::State::Maximized,
    );
    hit_test_geometry(maximized, location, window.geometry().size.w as f64, point)
}

pub fn hit_test_geometry(
    maximized: bool,
    location: Point<i32, Logical>,
    width: f64,
    point: Point<f64, Logical>,
) -> Option<Action> {
    let (y_min, y_max) = if maximized {
        (location.y as f64, location.y as f64 + HEIGHT as f64)
    } else {
        (location.y as f64 - HEIGHT as f64, location.y as f64)
    };
    if point.x < location.x as f64
        || point.x >= location.x as f64 + width
        || point.y < y_min
        || point.y >= y_max
        || (!maximized && outside_rounded_corner(location, width, y_min, point))
    {
        return None;
    }

    let close = location.x as f64 + width - 22.0;
    let max = close - 24.0;
    let min = max - 24.0;
    match true {
        _ if point.x >= close - BUTTON && point.x <= close + BUTTON => Some(Action::Close),
        _ if point.x >= max - BUTTON && point.x <= max + BUTTON => Some(Action::Maximize),
        _ if point.x >= min - BUTTON && point.x <= min + BUTTON => Some(Action::Minimize),
        _ => Some(Action::Move),
    }
}

fn outside_rounded_corner(
    location: Point<i32, Logical>,
    width: f64,
    y_min: f64,
    point: Point<f64, Logical>,
) -> bool {
    let center_x = match true {
        _ if point.x < location.x as f64 + CORNER_RADIUS => location.x as f64 + CORNER_RADIUS,
        _ if point.x > location.x as f64 + width - CORNER_RADIUS => {
            location.x as f64 + width - CORNER_RADIUS
        }
        _ => return false,
    };
    point.y < y_min + CORNER_RADIUS
        && (point.x - center_x).powi(2) + (point.y - (y_min + CORNER_RADIUS)).powi(2)
            > CORNER_RADIUS.powi(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floating_titlebar_actions_and_corner() {
        let location = Point::from((100, 200));
        let width = 800.0;
        assert_eq!(
            hit_test_geometry(false, location, width, Point::from((878.0, 180.0))),
            Some(Action::Close)
        );
        assert_eq!(
            hit_test_geometry(false, location, width, Point::from((854.0, 180.0))),
            Some(Action::Maximize)
        );
        assert_eq!(
            hit_test_geometry(false, location, width, Point::from((300.0, 150.0))),
            None
        );
        assert_eq!(
            hit_test_geometry(false, location, width, Point::from((102.0, 168.0))),
            None
        );
    }

    #[test]
    fn maximized_titlebar_actions_and_bounds() {
        let location = Point::from((0, 32));
        let width = 1920.0;
        assert_eq!(
            hit_test_geometry(true, location, width, Point::from((1898.0, 48.0))),
            Some(Action::Close)
        );
        assert_eq!(
            hit_test_geometry(true, location, width, Point::from((500.0, 20.0))),
            None
        );
        assert_eq!(
            hit_test_geometry(true, location, width, Point::from((500.0, 70.0))),
            None
        );
    }
}
