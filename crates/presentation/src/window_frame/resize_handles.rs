use domain::{CursorIcon, Point, Rect, Size};
use serde::{Deserialize, Serialize};

pub const DEFAULT_RESIZE_MARGIN: i32 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResizeDirection {
    Top,
    Bottom,
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl ResizeDirection {
    pub fn to_cursor_icon(self) -> CursorIcon {
        match self {
            Self::Top => CursorIcon::ResizeNorth,
            Self::Bottom => CursorIcon::ResizeSouth,
            Self::Left => CursorIcon::ResizeWest,
            Self::Right => CursorIcon::ResizeEast,
            Self::TopLeft => CursorIcon::ResizeNorthWest,
            Self::TopRight => CursorIcon::ResizeNorthEast,
            Self::BottomLeft => CursorIcon::ResizeSouthWest,
            Self::BottomRight => CursorIcon::ResizeSouthEast,
        }
    }

    /// Calculates new window geometry given an initial rectangle and mouse delta.
    pub fn apply_resize(&self, initial: Rect, dx: i32, dy: i32, min_size: Size) -> Rect {
        let mut x = initial.x();
        let mut y = initial.y();
        let mut w = initial.width() as i32;
        let mut h = initial.height() as i32;
        let min_w = min_size.width as i32;
        let min_h = min_size.height as i32;

        match self {
            Self::Right => {
                w = (w + dx).max(min_w);
            }
            Self::Bottom => {
                h = (h + dy).max(min_h);
            }
            Self::Left => {
                let new_w = (w - dx).max(min_w);
                x += w - new_w;
                w = new_w;
            }
            Self::Top => {
                let new_h = (h - dy).max(min_h);
                y += h - new_h;
                h = new_h;
            }
            Self::BottomRight => {
                w = (w + dx).max(min_w);
                h = (h + dy).max(min_h);
            }
            Self::BottomLeft => {
                let new_w = (w - dx).max(min_w);
                x += w - new_w;
                w = new_w;
                h = (h + dy).max(min_h);
            }
            Self::TopRight => {
                w = (w + dx).max(min_w);
                let new_h = (h - dy).max(min_h);
                y += h - new_h;
                h = new_h;
            }
            Self::TopLeft => {
                let new_w = (w - dx).max(min_w);
                let new_h = (h - dy).max(min_h);
                x += w - new_w;
                y += h - new_h;
                w = new_w;
                h = new_h;
            }
        }

        Rect::new(x, y, w as u32, h as u32)
    }
}

pub struct ResizeHandleDetector {
    pub margin: i32,
}

impl ResizeHandleDetector {
    pub fn new(margin: i32) -> Self {
        Self { margin }
    }

    pub fn hit_test(&self, window_rect: &Rect, point: Point) -> Option<ResizeDirection> {
        let m = self.margin;
        let x = window_rect.x();
        let y = window_rect.y();
        let r = window_rect.right();
        let b = window_rect.bottom();

        // Check if inside outer margin bounds
        let outer = Rect::new(
            x - m,
            y - m,
            window_rect.width() + (m * 2) as u32,
            window_rect.height() + (m * 2) as u32,
        );
        if !outer.contains_point(point) {
            return None;
        }

        let at_left = (point.x - x).abs() <= m;
        let at_right = (point.x - r).abs() <= m;
        let at_top = (point.y - y).abs() <= m;
        let at_bottom = (point.y - b).abs() <= m;

        if at_top && at_left {
            Some(ResizeDirection::TopLeft)
        } else if at_top && at_right {
            Some(ResizeDirection::TopRight)
        } else if at_bottom && at_left {
            Some(ResizeDirection::BottomLeft)
        } else if at_bottom && at_right {
            Some(ResizeDirection::BottomRight)
        } else if at_top {
            Some(ResizeDirection::Top)
        } else if at_bottom {
            Some(ResizeDirection::Bottom)
        } else if at_left {
            Some(ResizeDirection::Left)
        } else if at_right {
            Some(ResizeDirection::Right)
        } else {
            None
        }
    }
}

impl Default for ResizeHandleDetector {
    fn default() -> Self {
        Self::new(DEFAULT_RESIZE_MARGIN)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resize_handle_detection() {
        let detector = ResizeHandleDetector::default();
        let win = Rect::new(100, 100, 400, 300);

        // Near top-left
        assert_eq!(
            detector.hit_test(&win, Point::new(100, 100)),
            Some(ResizeDirection::TopLeft)
        );

        // Near bottom-right (right=500, bottom=400)
        assert_eq!(
            detector.hit_test(&win, Point::new(500, 400)),
            Some(ResizeDirection::BottomRight)
        );

        // Center of window -> not a resize handle
        assert_eq!(detector.hit_test(&win, Point::new(300, 250)), None);
    }
}
