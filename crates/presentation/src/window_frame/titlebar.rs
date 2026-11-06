use domain::{Color, Point, Rect};
use serde::{Deserialize, Serialize};

pub const DEFAULT_TITLEBAR_HEIGHT: u32 = 32;
pub const DEFAULT_BUTTON_WIDTH: u32 = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TitlebarButton {
    Minimize,
    Maximize,
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TitlebarHit {
    Button(TitlebarButton),
    DragTitlebar,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TitlebarTheme {
    pub height: u32,
    pub button_width: u32,
    pub background_focused: Color,
    pub background_unfocused: Color,
    pub text_color_focused: Color,
    pub text_color_unfocused: Color,
    pub close_button_hover: Color,
}

impl Default for TitlebarTheme {
    fn default() -> Self {
        Self {
            height: DEFAULT_TITLEBAR_HEIGHT,
            button_width: DEFAULT_BUTTON_WIDTH,
            background_focused: Color::rgb(35, 38, 52),
            background_unfocused: Color::rgb(24, 25, 38),
            text_color_focused: Color::WHITE,
            text_color_unfocused: Color::rgb(140, 145, 160),
            close_button_hover: Color::rgb(235, 64, 52),
        }
    }
}

pub struct TitlebarDecorator {
    pub theme: TitlebarTheme,
}

impl TitlebarDecorator {
    pub fn new(theme: TitlebarTheme) -> Self {
        Self { theme }
    }

    pub fn titlebar_rect(&self, window_rect: &Rect) -> Rect {
        Rect::new(
            window_rect.x(),
            window_rect.y(),
            window_rect.width(),
            self.theme.height,
        )
    }

    pub fn client_rect(&self, window_rect: &Rect) -> Rect {
        let titlebar_h = self.theme.height;
        let client_h = window_rect.height().saturating_sub(titlebar_h);
        Rect::new(
            window_rect.x(),
            window_rect.y() + titlebar_h as i32,
            window_rect.width(),
            client_h,
        )
    }

    pub fn button_rect(&self, window_rect: &Rect, button: TitlebarButton) -> Rect {
        let btn_w = self.theme.button_width;
        let btn_h = self.theme.height;
        let right = window_rect.right();
        let top = window_rect.y();

        let x = match button {
            TitlebarButton::Close => right - btn_w as i32,
            TitlebarButton::Maximize => right - (btn_w * 2) as i32,
            TitlebarButton::Minimize => right - (btn_w * 3) as i32,
        };

        Rect::new(x, top, btn_w, btn_h)
    }

    pub fn hit_test(&self, window_rect: &Rect, point: Point) -> TitlebarHit {
        let bar = self.titlebar_rect(window_rect);
        if !bar.contains_point(point) {
            return TitlebarHit::None;
        }

        if self
            .button_rect(window_rect, TitlebarButton::Close)
            .contains_point(point)
        {
            return TitlebarHit::Button(TitlebarButton::Close);
        }
        if self
            .button_rect(window_rect, TitlebarButton::Maximize)
            .contains_point(point)
        {
            return TitlebarHit::Button(TitlebarButton::Maximize);
        }
        if self
            .button_rect(window_rect, TitlebarButton::Minimize)
            .contains_point(point)
        {
            return TitlebarHit::Button(TitlebarButton::Minimize);
        }

        TitlebarHit::DragTitlebar
    }
}

impl Default for TitlebarDecorator {
    fn default() -> Self {
        Self::new(TitlebarTheme::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_titlebar_geometry_and_hits() {
        let decorator = TitlebarDecorator::default();
        let win = Rect::new(100, 100, 800, 600);

        let bar = decorator.titlebar_rect(&win);
        assert_eq!(bar, Rect::new(100, 100, 800, 32));

        let client = decorator.client_rect(&win);
        assert_eq!(client, Rect::new(100, 132, 800, 568));

        // Drag hit
        let drag_point = Point::new(200, 110);
        assert_eq!(
            decorator.hit_test(&win, drag_point),
            TitlebarHit::DragTitlebar
        );

        // Close button hit (right: 900, close btn: [868..900])
        let close_point = Point::new(880, 110);
        assert_eq!(
            decorator.hit_test(&win, close_point),
            TitlebarHit::Button(TitlebarButton::Close)
        );

        // Outside titlebar
        let client_point = Point::new(200, 200);
        assert_eq!(decorator.hit_test(&win, client_point), TitlebarHit::None);
    }
}
