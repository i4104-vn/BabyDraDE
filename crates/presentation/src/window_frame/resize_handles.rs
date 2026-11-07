use domain::{Point, Rect};
pub use domain::ResizeDirection;

pub const DEFAULT_RESIZE_MARGIN: i32 = 8;

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
