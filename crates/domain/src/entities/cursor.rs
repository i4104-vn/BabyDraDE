use crate::value_objects::Point;
use serde::{Deserialize, Serialize};

/// Cursor icon variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum CursorIcon {
    #[default]
    Default,
    Pointer,
    Grab,
    Grabbing,
    Text,
    Crosshair,
    Move,
    NotAllowed,
    Wait,
    ResizeNorth,
    ResizeSouth,
    ResizeEast,
    ResizeWest,
    ResizeNorthEast,
    ResizeNorthWest,
    ResizeSouthEast,
    ResizeSouthWest,
}

/// Software / hardware pointer cursor state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cursor {
    pub position: Point,
    pub icon: CursorIcon,
    pub visible: bool,
}

impl Cursor {
    pub const fn new(position: Point) -> Self {
        Self {
            position,
            icon: CursorIcon::Default,
            visible: true,
        }
    }

    pub fn set_position(&mut self, p: Point) {
        self.position = p;
    }

    pub fn move_by(&mut self, dx: i32, dy: i32) {
        self.position = self.position.translated(dx, dy);
    }

    pub fn set_icon(&mut self, icon: CursorIcon) {
        self.icon = icon;
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self::new(Point::ZERO)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_movement() {
        let mut cursor = Cursor::new(Point::new(100, 200));
        cursor.move_by(15, -20);
        assert_eq!(cursor.position, Point::new(115, 180));

        cursor.set_icon(CursorIcon::ResizeNorthWest);
        assert_eq!(cursor.icon, CursorIcon::ResizeNorthWest);
    }
}
