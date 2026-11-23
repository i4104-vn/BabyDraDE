//! Titlebar geometry and hit testing (pure rules).
//!
//! Geometry primitives from `smithay::utils` are used as value objects; no
//! window handle, protocol state or I/O is involved. The adapter extracts the
//! frame facts into [`FrameInput`] before calling [`hit_test`], so renderer and
//! input path can never drift apart.

use common::{Action, CORNER_RADIUS, TITLEBAR_BUTTON_RADIUS, TITLEBAR_HEIGHT};
use smithay::utils::{Logical, Point};

/// Protocol-independent facts about a window frame.
///
/// The adapter extracts these from a window so hit testing can stay a pure rule.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct FrameInput {
    pub(crate) location: Point<i32, Logical>,
    pub(crate) width: f64,
    pub(crate) maximized: bool,
    pub(crate) server_side_decoration: bool,
    pub(crate) fullscreen: bool,
}

/// Hit-tests a pointer position against a compositor-owned frame.
///
/// # Arguments
///
/// * `frame` - Frame geometry and decoration state.
/// * `point` - Pointer position in global logical coordinates.
///
/// # Returns
///
/// The action under the pointer, or `None` when the point is outside a frame
/// that can be hit tested.
pub(crate) fn hit_test(frame: FrameInput, point: Point<f64, Logical>) -> Option<Action> {
    if !frame.server_side_decoration || frame.fullscreen {
        return None;
    }
    TitlebarLayout::new(frame.width, frame.maximized).hit_test(
        frame.location,
        frame.width,
        frame.maximized,
        point,
    )
}

/// A circular titlebar button.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct TitlebarButton {
    pub(crate) action: Action,
    /// Center in titlebar-local logical coordinates.
    pub(crate) center: Point<f64, Logical>,
}

/// Geometry of the compositor-owned titlebar for a given window width.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TitlebarLayout {
    pub(crate) height: i32,
    pub(crate) button_radius: f64,
    pub(crate) corner_radius: f64,
    pub(crate) buttons: [TitlebarButton; 3],
}

impl TitlebarLayout {
    /// Builds the titlebar layout for a given window width.
    ///
    /// # Arguments
    ///
    /// * `width` - Window width in logical pixels.
    /// * `_maximized` - Reserved for future layout variants.
    ///
    /// # Returns
    ///
    /// The button geometry for this width.
    pub(crate) fn new(width: f64, _maximized: bool) -> Self {
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

    /// Returns the centre of a titlebar button.
    ///
    /// # Arguments
    ///
    /// * `action` - Button action to look up.
    ///
    /// # Returns
    ///
    /// The button centre, or the origin when the action has no button.
    pub(crate) fn button(&self, action: Action) -> Point<f64, Logical> {
        self.buttons
            .iter()
            .find(|button| button.action == action)
            .map(|button| button.center)
            .unwrap_or_default()
    }

    /// Computes the titlebar's top edge in global logical coordinates.
    ///
    /// # Arguments
    ///
    /// * `location` - Window location in logical pixels.
    /// * `maximized` - Whether the window is maximized.
    ///
    /// # Returns
    ///
    /// The Y coordinate of the frame's top edge.
    pub(crate) fn frame_y(&self, location: Point<i32, Logical>, maximized: bool) -> f64 {
        if maximized {
            location.y as f64
        } else {
            location.y as f64 - self.height as f64
        }
    }

    /// Hit-tests this layout against a point.
    ///
    /// # Arguments
    ///
    /// * `location` - Window location in logical pixels.
    /// * `width` - Window width in logical pixels.
    /// * `maximized` - Whether the window is maximized.
    /// * `point` - Pointer position in global logical coordinates.
    ///
    /// # Returns
    ///
    /// The matching button action, `Action::Move` on the draggable titlebar,
    /// or `None` outside the frame.
    pub(crate) fn hit_test(
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
        let frame = FrameInput {
            location: Point::from((100, 200)),
            width: 800.0,
            maximized: false,
            server_side_decoration: true,
            fullscreen: false,
        };
        let close = TitlebarLayout::new(frame.width, false).button(Action::Close);
        let point = Point::from((frame.location.x as f64 + close.x, 180.0));
        assert_eq!(hit_test(frame, point), Some(Action::Close));
    }

    #[test]
    fn client_decorations_and_fullscreen_have_no_frame() {
        let mut frame = FrameInput {
            location: Point::from((0, 0)),
            width: 800.0,
            maximized: false,
            server_side_decoration: false,
            fullscreen: false,
        };
        assert_eq!(hit_test(frame, Point::from((750.0, 10.0))), None);
        frame.server_side_decoration = true;
        frame.fullscreen = true;
        assert_eq!(hit_test(frame, Point::from((750.0, 10.0))), None);
    }
}
