//! Shared screen-edge dock used by the launcher (left) and the switcher
//! (right). Geometry, selection/animation state and the capsule rendering are
//! split into small modules; callers provide only their side content.

mod draw;
mod layout;
mod state;

pub use common::DockSide;
pub use draw::draw_dock;
pub use layout::{visual_center, DockLayout};
pub use state::DockState;

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn dock_is_placed_on_the_requested_edge() {
        let left = DockLayout::new(5, 400.0, 1920.0, DockSide::Left);
        let right = DockLayout::new(5, 400.0, 1920.0, DockSide::Right);
        assert!(left.x < right.x);
        assert_eq!(right.x + right.width, 1920.0);
    }

    #[test]
    fn short_lists_wrap_the_selection() {
        assert_eq!(visual_center(5, 7.0), 2.0);
        let mut state = DockState::new();
        state.open(4, false);
        assert_eq!(state.selected, Some(4));
        state.move_selection(5, 1, Duration::ZERO);
        assert_eq!(state.selected, Some(4));
    }
}
