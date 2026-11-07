use serde::{Deserialize, Serialize};

/// A 2D point with integer coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const ZERO: Self = Self { x: 0, y: 0 };

    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn translated(self, dx: i32, dy: i32) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
    }

    pub fn distance_squared_to(self, other: Point) -> i64 {
        let dx = (self.x - other.x) as i64;
        let dy = (self.y - other.y) as i64;
        dx * dx + dy * dy
    }
}

/// A 2D size with non-negative dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub const ZERO: Self = Self {
        width: 0,
        height: 0,
    };

    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub fn area(self) -> u64 {
        self.width as u64 * self.height as u64
    }

    pub fn is_empty(self) -> bool {
        self.width == 0 || self.height == 0
    }
}

/// Padding or margins for 4 edges (top, right, bottom, left).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Edges {
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
    pub left: u32,
}

impl Edges {
    pub const ZERO: Self = Self {
        top: 0,
        right: 0,
        bottom: 0,
        left: 0,
    };

    pub const fn new(top: u32, right: u32, bottom: u32, left: u32) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    pub const fn all(value: u32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    pub const fn symmetric(vertical: u32, horizontal: u32) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    pub fn horizontal(self) -> u32 {
        self.left + self.right
    }

    pub fn vertical(self) -> u32 {
        self.top + self.bottom
    }
}

/// A 2D axis-aligned rectangle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Rect {
    pub origin: Point,
    pub size: Size,
}

impl Rect {
    pub const ZERO: Self = Self {
        origin: Point::ZERO,
        size: Size::ZERO,
    };

    pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            origin: Point::new(x, y),
            size: Size::new(width, height),
        }
    }

    pub const fn from_point_and_size(origin: Point, size: Size) -> Self {
        Self { origin, size }
    }

    pub const fn x(&self) -> i32 {
        self.origin.x
    }

    pub const fn y(&self) -> i32 {
        self.origin.y
    }

    pub const fn width(&self) -> u32 {
        self.size.width
    }

    pub const fn height(&self) -> u32 {
        self.size.height
    }

    pub fn right(&self) -> i32 {
        self.origin.x + self.size.width as i32
    }

    pub fn bottom(&self) -> i32 {
        self.origin.y + self.size.height as i32
    }

    pub fn contains_point(&self, point: Point) -> bool {
        point.x >= self.x()
            && point.x < self.right()
            && point.y >= self.y()
            && point.y < self.bottom()
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        self.x() < other.right()
            && self.right() > other.x()
            && self.y() < other.bottom()
            && self.bottom() > other.y()
    }

    pub fn intersection(&self, other: &Rect) -> Option<Rect> {
        let x1 = self.x().max(other.x());
        let y1 = self.y().max(other.y());
        let x2 = self.right().min(other.right());
        let y2 = self.bottom().min(other.bottom());

        if x1 < x2 && y1 < y2 {
            Some(Rect::new(x1, y1, (x2 - x1) as u32, (y2 - y1) as u32))
        } else {
            None
        }
    }

    pub fn union(&self, other: &Rect) -> Rect {
        if self.size.is_empty() {
            return *other;
        }
        if other.size.is_empty() {
            return *self;
        }

        let x1 = self.x().min(other.x());
        let y1 = self.y().min(other.y());
        let x2 = self.right().max(other.right());
        let y2 = self.bottom().max(other.bottom());

        Rect::new(x1, y1, (x2 - x1) as u32, (y2 - y1) as u32)
    }

    pub fn translated(&self, dx: i32, dy: i32) -> Rect {
        Rect {
            origin: self.origin.translated(dx, dy),
            size: self.size,
        }
    }

    pub fn resized(&self, width: u32, height: u32) -> Rect {
        Rect {
            origin: self.origin,
            size: Size::new(width, height),
        }
    }

    pub fn inset(&self, edges: &Edges) -> Rect {
        let h_edges = edges.horizontal();
        let v_edges = edges.vertical();

        let width = self.size.width.saturating_sub(h_edges);
        let height = self.size.height.saturating_sub(v_edges);

        Rect::new(
            self.origin.x + edges.left as i32,
            self.origin.y + edges.top as i32,
            width,
            height,
        )
    }
}

/// Target positions for window snapping / tiling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SnapPosition {
    LeftHalf,
    RightHalf,
    TopHalf,
    BottomHalf,
}

/// Directions for interactive window edge/corner resizing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    pub fn to_cursor_icon(self) -> crate::entities::cursor::CursorIcon {
        use crate::entities::cursor::CursorIcon;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_operations() {
        let p = Point::new(10, 20);
        assert_eq!(p.translated(5, -5), Point::new(15, 15));
        assert_eq!(p.distance_squared_to(Point::new(13, 24)), 9 + 16);
    }

    #[test]
    fn test_size_operations() {
        let s = Size::new(800, 600);
        assert_eq!(s.area(), 480_000);
        assert!(!s.is_empty());
        assert!(Size::ZERO.is_empty());
    }

    #[test]
    fn test_rect_intersection_and_contains() {
        let r1 = Rect::new(0, 0, 100, 100);
        let r2 = Rect::new(50, 50, 100, 100);
        let r3 = Rect::new(200, 200, 50, 50);

        assert!(r1.contains_point(Point::new(50, 50)));
        assert!(!r1.contains_point(Point::new(100, 100)));

        assert!(r1.intersects(&r2));
        assert!(!r1.intersects(&r3));

        let inter = r1.intersection(&r2).unwrap();
        assert_eq!(inter, Rect::new(50, 50, 50, 50));
        assert_eq!(r1.intersection(&r3), None);

        let un = r1.union(&r2);
        assert_eq!(un, Rect::new(0, 0, 150, 150));
    }

    #[test]
    fn test_rect_inset() {
        let r = Rect::new(10, 10, 100, 100);
        let edges = Edges::new(5, 10, 15, 20);
        let inset = r.inset(&edges);
        assert_eq!(inset, Rect::new(30, 15, 70, 80));
    }
}
