use crate::value_objects::{Color, Point, Rect};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderError {
    ContextLost,
    TextureCreationFailed(String),
    DrawingFailed(String),
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ContextLost => write!(f, "Render context was lost"),
            Self::TextureCreationFailed(msg) => write!(f, "Failed to create GPU texture: {msg}"),
            Self::DrawingFailed(msg) => write!(f, "Render drawing operation failed: {msg}"),
        }
    }
}

impl std::error::Error for RenderError {}

/// Contract for Canvas and scene rendering (OpenGL ES / Software / Mock).
pub trait CanvasRenderer {
    /// Clears the background canvas with a solid color.
    fn clear(&mut self, color: Color) -> Result<(), RenderError>;

    /// Draws a solid rectangle (used for borders, titlebars, background elements).
    fn draw_rect(&mut self, rect: Rect, color: Color) -> Result<(), RenderError>;

    /// Draws text at a given position.
    fn draw_text(
        &mut self,
        text: &str,
        position: Point,
        font_size: f32,
        color: Color,
    ) -> Result<(), RenderError>;

    /// Flushes and presents rendered frame to display.
    fn render_frame(&mut self) -> Result<(), RenderError>;
}
