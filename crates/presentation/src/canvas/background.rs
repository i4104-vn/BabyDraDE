use domain::{CanvasRenderer, Color, Rect, RenderError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackgroundStyle {
    SolidColor(Color),
    Wallpaper { path: String, fallback: Color },
}

impl Default for BackgroundStyle {
    fn default() -> Self {
        // Deep modern dark background (e.g. #1e1e2e)
        Self::SolidColor(Color::rgb(30, 30, 46))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanvasBackground {
    pub style: BackgroundStyle,
}

impl CanvasBackground {
    pub fn new(style: BackgroundStyle) -> Self {
        Self { style }
    }

    pub fn render(
        &self,
        renderer: &mut dyn CanvasRenderer,
        screen_rect: Rect,
    ) -> Result<(), RenderError> {
        match &self.style {
            BackgroundStyle::SolidColor(color) => {
                renderer.clear(*color)?;
                renderer.draw_rect(screen_rect, *color)?;
            }
            BackgroundStyle::Wallpaper { fallback, .. } => {
                // If texture wallpaper isn't loaded yet, clear with fallback
                renderer.clear(*fallback)?;
                renderer.draw_rect(screen_rect, *fallback)?;
            }
        }
        Ok(())
    }
}

impl Default for CanvasBackground {
    fn default() -> Self {
        Self::new(BackgroundStyle::default())
    }
}
