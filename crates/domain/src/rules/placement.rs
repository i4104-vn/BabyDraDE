use crate::value_objects::{Point, Rect, Size};
use serde::{Deserialize, Serialize};

/// Default offset in pixels for cascade placement.
pub const DEFAULT_CASCADE_OFFSET: i32 = 30;

/// Policy for placing newly opened windows on a screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlacementPolicy {
    Center,
    Cascade { offset: i32 },
}

impl Default for PlacementPolicy {
    fn default() -> Self {
        Self::Cascade {
            offset: DEFAULT_CASCADE_OFFSET,
        }
    }
}

impl PlacementPolicy {
    pub fn place(&self, screen: Rect, window_size: Size, last_window_rect: Option<Rect>) -> Rect {
        match *self {
            PlacementPolicy::Center => place_center(screen, window_size),
            PlacementPolicy::Cascade { offset } => {
                place_cascade(screen, window_size, last_window_rect, offset)
            }
        }
    }
}

/// Computes the geometry to place a window directly at the center of the screen.
pub fn place_center(screen: Rect, window_size: Size) -> Rect {
    let screen_w = screen.width() as i32;
    let screen_h = screen.height() as i32;
    let win_w = window_size.width as i32;
    let win_h = window_size.height as i32;

    let x = screen.x() + (screen_w - win_w) / 2;
    let y = screen.y() + (screen_h - win_h) / 2;

    Rect::from_point_and_size(Point::new(x, y), window_size)
}

/// Computes cascade geometry (shifted down-right by offset px from previous window).
/// Wraps around to screen origin + offset if the window exceeds the screen boundary.
pub fn place_cascade(
    screen: Rect,
    window_size: Size,
    last_window_rect: Option<Rect>,
    offset: i32,
) -> Rect {
    let offset = if offset <= 0 {
        DEFAULT_CASCADE_OFFSET
    } else {
        offset
    };
    let initial_x = screen.x() + offset;
    let initial_y = screen.y() + offset;

    let (target_x, target_y) = match last_window_rect {
        Some(last) => {
            let next_x = last.x() + offset;
            let next_y = last.y() + offset;

            let exceeds_right = next_x + window_size.width as i32 > screen.right();
            let exceeds_bottom = next_y + window_size.height as i32 > screen.bottom();

            if exceeds_right || exceeds_bottom {
                (initial_x, initial_y)
            } else {
                (next_x, next_y)
            }
        }
        None => (initial_x, initial_y),
    };

    Rect::from_point_and_size(Point::new(target_x, target_y), window_size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_place_center() {
        let screen = Rect::new(0, 0, 1920, 1080);
        let win_size = Size::new(800, 600);
        let rect = place_center(screen, win_size);

        assert_eq!(rect.origin.x, (1920 - 800) / 2);
        assert_eq!(rect.origin.y, (1080 - 600) / 2);
        assert_eq!(rect.size, win_size);
    }

    #[test]
    fn test_place_cascade_initial_and_step() {
        let screen = Rect::new(0, 0, 1000, 800);
        let size = Size::new(400, 300);

        // First window
        let first = place_cascade(screen, size, None, 30);
        assert_eq!(first, Rect::new(30, 30, 400, 300));

        // Second window cascades
        let second = place_cascade(screen, size, Some(first), 30);
        assert_eq!(second, Rect::new(60, 60, 400, 300));
    }

    #[test]
    fn test_place_cascade_wrap_around() {
        let screen = Rect::new(0, 0, 1000, 800);
        let size = Size::new(400, 300);

        // Window placed near bottom-right edge
        let near_edge = Rect::new(580, 480, 400, 300);
        let wrapped = place_cascade(screen, size, Some(near_edge), 30);

        assert_eq!(wrapped, Rect::new(30, 30, 400, 300));
    }
}
