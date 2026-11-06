use domain::{ButtonState, Cursor, CursorIcon, Point};

/// Tracks physical pointer device state and cursor properties.
#[derive(Debug, Default)]
pub struct PointerDevice {
    pub cursor: Cursor,
    pub left_button_down: bool,
    pub right_button_down: bool,
    pub middle_button_down: bool,
}

impl PointerDevice {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn on_motion(&mut self, dx: i32, dy: i32) -> Point {
        self.cursor.move_by(dx, dy);
        self.cursor.position
    }

    pub fn set_position(&mut self, pos: Point) {
        self.cursor.set_position(pos);
    }

    pub fn on_button(&mut self, button: u32, state: ButtonState) {
        let is_down = state == ButtonState::Pressed;
        match button {
            1 | 272 => self.left_button_down = is_down,   // BTN_LEFT / Linux 272
            2 | 273 => self.right_button_down = is_down,  // BTN_RIGHT / Linux 273
            3 | 274 => self.middle_button_down = is_down, // BTN_MIDDLE / Linux 274
            _ => {}
        }
    }

    pub fn set_cursor_icon(&mut self, icon: CursorIcon) {
        self.cursor.set_icon(icon);
    }
}
