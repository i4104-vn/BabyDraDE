use domain::{CanvasRenderer, Color, Point, Rect, RenderError};
use crate::canvas::background::CanvasBackground;

/// Fullscreen Desktop Canvas for BabyDraDE.
/// Acts as the core visual canvas on which all floating window textures,
/// server-side titlebars, borders, overlays, and cursors are composed.
#[derive(Debug, Clone)]
pub struct DesktopCanvas {
    pub bounds: Rect,
    pub background: CanvasBackground,
}

impl DesktopCanvas {
    /// Creates a new desktop canvas with specified screen bounds.
    pub fn new(bounds: Rect, background: CanvasBackground) -> Self {
        Self { bounds, background }
    }

    /// Renders the base canvas surface (solid color or wallpaper texture).
    pub fn render_base(&self, renderer: &mut dyn CanvasRenderer) -> Result<(), RenderError> {
        self.background.render(renderer, self.bounds)
    }

    /// Draws a rectangular element directly onto the canvas surface.
    pub fn draw_rect(
        &self,
        renderer: &mut dyn CanvasRenderer,
        rect: Rect,
        color: Color,
    ) -> Result<(), RenderError> {
        renderer.draw_rect(rect, color)
    }

    /// Draws text onto the canvas surface.
    pub fn draw_text(
        &self,
        renderer: &mut dyn CanvasRenderer,
        text: &str,
        position: Point,
        font_size: f32,
        color: Color,
    ) -> Result<(), RenderError> {
        renderer.draw_text(text, position, font_size, color)
    }

    /// Clears the canvas surface with a solid color.
    pub fn clear(
        &self,
        renderer: &mut dyn CanvasRenderer,
        color: Color,
    ) -> Result<(), RenderError> {
        renderer.clear(color)
    }
}

impl Default for DesktopCanvas {
    fn default() -> Self {
        Self {
            bounds: Rect::new(0, 0, 1920, 1080),
            background: CanvasBackground::default(),
        }
    }
}

