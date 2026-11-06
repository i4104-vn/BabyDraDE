use domain::{Color, Rect};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowTheme {
    pub enabled: bool,
    pub blur_radius: u32,
    pub spread: u32,
    pub color: Color,
    pub offset_x: i32,
    pub offset_y: i32,
}

impl Default for ShadowTheme {
    fn default() -> Self {
        Self {
            enabled: true,
            blur_radius: 16,
            spread: 2,
            color: Color::rgba(0, 0, 0, 100),
            offset_x: 0,
            offset_y: 6,
        }
    }
}

impl ShadowTheme {
    /// Computes shadow boundary rect around window.
    pub fn shadow_rect(&self, window_rect: &Rect) -> Option<Rect> {
        if !self.enabled {
            return None;
        }

        let pad = (self.blur_radius + self.spread) as i32;
        let x = window_rect.x() + self.offset_x - pad;
        let y = window_rect.y() + self.offset_y - pad;
        let w = window_rect.width() + (pad * 2) as u32;
        let h = window_rect.height() + (pad * 2) as u32;

        Some(Rect::new(x, y, w, h))
    }
}
