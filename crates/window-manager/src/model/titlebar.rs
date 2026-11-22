//! Shared titlebar geometry and hit testing.
//!
//! Both the renderer and the input hit-test path use this layout so button
//! positions can never drift apart.

use common::{Action, CORNER_RADIUS, TITLEBAR_BUTTON_RADIUS, TITLEBAR_HEIGHT};
use smithay::utils::{Logical, Point};

/// A circular titlebar button.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TitlebarButton {
    pub action: Action,
    /// Center in titlebar-local logical coordinates.
    pub center: Point<f64, Logical>,
}

/// Geometry of the compositor-owned titlebar for a given window width.
#[derive(Debug, Clone, PartialEq)]
pub struct TitlebarLayout {
    pub height: i32,
    pub button_radius: f64,
    pub corner_radius: f64,
    pub buttons: [TitlebarButton; 3],
}

impl TitlebarLayout {
    /// Builds the layout for a window of `width` logical pixels.
    pub fn new(width: f64, _maximized: bool) -> Self {
        let cy = TITLEBAR_HEIGHT as f64 / 2.0;
        let close_x = width - 22.0;
        let max_x = close_x - 24.0;
        let min_x = max_x - 24.0;

        Self {
            height: TITLEBAR_HEIGHT,
            button_radius: TITLEBAR_BUTTON_RADIUS,
            corner_radius: CORNER_RADIUS,
            buttons: [
                TitlebarButton {
                    action: Action::Close,
                    center: Point::from((close_x, cy)),
                },
                TitlebarButton {
                    action: Action::Maximize,
                    center: Point::from((max_x, cy)),
                },
                TitlebarButton {
                    action: Action::Minimize,
                    center: Point::from((min_x, cy)),
                },
            ],
        }
    }

    /// Returns the center of a button, or the origin if the action is unknown.
    pub fn button(&self, action: Action) -> Point<f64, Logical> {
        self.buttons
            .iter()
            .find(|button| button.action == action)
            .map(|button| button.center)
            .unwrap_or_default()
    }

    /// Logical Y coordinate of the titlebar top edge.
    pub fn frame_y(&self, location: Point<i32, Logical>, maximized: bool) -> f64 {
        if maximized {
            location.y as f64
        } else {
            location.y as f64 - self.height as f64
        }
    }

    /// Hit-tests a point in global logical coordinates.
    pub fn hit_test(
        &self,
        location: Point<i32, Logical>,
        width: f64,
        maximized: bool,
        point: Point<f64, Logical>,
    ) -> Option<Action> {
        let (y_min, y_max) = if maximized {
            (location.y as f64, location.y as f64 + self.height as f64)
        } else {
            (location.y as f64 - self.height as f64, location.y as f64)
        };

        if point.x < location.x as f64
            || point.x >= location.x as f64 + width
            || point.y < y_min
            || point.y >= y_max
            || (!maximized && self.outside_rounded_corner(location, width, y_min, point))
        {
            return None;
        }

        for button in &self.buttons {
            let x = location.x as f64 + button.center.x;
            if (point.x - x).abs() <= self.button_radius {
                return Some(button.action);
            }
        }
        Some(Action::Move)
    }

    fn outside_rounded_corner(
        &self,
        location: Point<i32, Logical>,
        width: f64,
        y_min: f64,
        point: Point<f64, Logical>,
    ) -> bool {
        let center_x = if point.x < location.x as f64 + self.corner_radius {
            location.x as f64 + self.corner_radius
        } else if point.x > location.x as f64 + width - self.corner_radius {
            location.x as f64 + width - self.corner_radius
        } else {
            return false;
        };
        point.y < y_min + self.corner_radius
            && (point.x - center_x).powi(2) + (point.y - (y_min + self.corner_radius)).powi(2)
                > self.corner_radius.powi(2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layouts_share_button_positions() {
        let layout = TitlebarLayout::new(800.0, false);
        let close = layout.button(Action::Close);
        let max = layout.button(Action::Maximize);
        let min = layout.button(Action::Minimize);
        assert_eq!(close.x - max.x, 24.0);
        assert_eq!(max.x - min.x, 24.0);
        assert_eq!(close.y, max.y);
        assert_eq!(max.y, min.y);
    }

    #[test]
    fn hit_test_matches_layout_buttons() {
        let layout = TitlebarLayout::new(800.0, false);
        let location = Point::from((100, 200));
        let close = layout.button(Action::Close);
        let point = Point::from((location.x as f64 + close.x, 180.0));
        assert_eq!(
            layout.hit_test(location, 800.0, false, point),
            Some(Action::Close)
        );
    }
}
