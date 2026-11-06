use domain::{Color, Edges, Rect};
use serde::{Deserialize, Serialize};

pub const DEFAULT_BORDER_WIDTH: u32 = 2;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BorderTheme {
    pub width: u32,
    pub color_focused: Color,
    pub color_unfocused: Color,
}

impl Default for BorderTheme {
    fn default() -> Self {
        Self {
            width: DEFAULT_BORDER_WIDTH,
            color_focused: Color::rgb(137, 180, 250), // Catppuccin / vibrant accent
            color_unfocused: Color::rgb(49, 50, 68),  // Subtle dark
        }
    }
}

pub struct BorderDecorator {
    pub theme: BorderTheme,
}

impl BorderDecorator {
    pub fn new(theme: BorderTheme) -> Self {
        Self { theme }
    }

    pub fn border_color(&self, focused: bool) -> Color {
        if focused {
            self.theme.color_focused
        } else {
            self.theme.color_unfocused
        }
    }

    pub fn edges(&self) -> Edges {
        Edges::all(self.theme.width)
    }

    /// Computes the 4 border rectangles (top, bottom, left, right) to render around a window frame.
    pub fn border_rects(&self, window_rect: &Rect) -> [Rect; 4] {
        let b = self.theme.width;
        let x = window_rect.x();
        let y = window_rect.y();
        let w = window_rect.width();
        let h = window_rect.height();

        let top = Rect::new(x, y, w, b);
        let bottom = Rect::new(x, y + h as i32 - b as i32, w, b);
        let left = Rect::new(x, y + b as i32, b, h.saturating_sub(b * 2));
        let right = Rect::new(
            x + w as i32 - b as i32,
            y + b as i32,
            b,
            h.saturating_sub(b * 2),
        );

        [top, bottom, left, right]
    }
}

impl Default for BorderDecorator {
    fn default() -> Self {
        Self::new(BorderTheme::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_border_rects() {
        let bd = BorderDecorator::default();
        let win = Rect::new(100, 100, 400, 300);
        let rects = bd.border_rects(&win);

        assert_eq!(rects[0], Rect::new(100, 100, 400, 2)); // top
        assert_eq!(rects[1], Rect::new(100, 398, 400, 2)); // bottom
    }
}
