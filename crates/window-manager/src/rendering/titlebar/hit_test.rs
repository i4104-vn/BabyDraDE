//! Titlebar pointer hit testing.

use smithay::{
    desktop::Window,
    utils::{Logical, Point},
};

use common::Action;

use crate::model::TitlebarLayout;

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
    let width = window.geometry().size.w as f64;
    TitlebarLayout::new(width, maximized).hit_test(location, width, maximized, point)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floating_titlebar_actions_and_corner() {
        let location = Point::from((100, 200));
        let width = 800.0;
        let layout = TitlebarLayout::new(width, false);
        assert_eq!(
            layout.hit_test(location, width, false, Point::from((878.0, 180.0))),
            Some(Action::Close)
        );
        assert_eq!(
            layout.hit_test(location, width, false, Point::from((854.0, 180.0))),
            Some(Action::Maximize)
        );
        assert_eq!(
            layout.hit_test(location, width, false, Point::from((300.0, 150.0))),
            None
        );
        assert_eq!(
            layout.hit_test(location, width, false, Point::from((102.0, 168.0))),
            None
        );
    }

    #[test]
    fn maximized_titlebar_actions_and_bounds() {
        let location = Point::from((0, 32));
        let width = 1920.0;
        let layout = TitlebarLayout::new(width, true);
        assert_eq!(
            layout.hit_test(location, width, true, Point::from((1898.0, 48.0))),
            Some(Action::Close)
        );
        assert_eq!(
            layout.hit_test(location, width, true, Point::from((500.0, 20.0))),
            None
        );
        assert_eq!(
            layout.hit_test(location, width, true, Point::from((500.0, 70.0))),
            None
        );
    }
}
