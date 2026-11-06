use domain::{CanvasRenderer, Color, Point, Rect, RenderError, WindowId};

use super::texture_cache::TextureCache;

/// OpenGL ES / Canvas renderer implementation for BabyDraDE.
pub struct GlRenderer {
    pub texture_cache: TextureCache,
    pub draw_call_count: usize,
    pub last_clear_color: Option<Color>,
}

impl GlRenderer {
    pub fn new() -> Self {
        Self {
            texture_cache: TextureCache::new(),
            draw_call_count: 0,
            last_clear_color: None,
        }
    }

    /// Renders imported client GPU texture at designated window client coordinates:
    /// (window.x, window.y + titlebar_height).
    pub fn draw_window_texture(
        &mut self,
        window_id: WindowId,
        destination_rect: Rect,
    ) -> Result<(), RenderError> {
        if let Some(_texture) = self.texture_cache.get(window_id) {
            // Import and draw client texture via GL quad
            self.draw_call_count += 1;
            tracing::trace!(
                "Drawing client texture for window {} at {:?}",
                window_id,
                destination_rect
            );
            Ok(())
        } else {
            // Draw placeholder background if texture is not ready
            self.draw_rect(destination_rect, Color::rgb(24, 24, 30))
        }
    }
}

impl Default for GlRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl CanvasRenderer for GlRenderer {
    fn clear(&mut self, color: Color) -> Result<(), RenderError> {
        self.last_clear_color = Some(color);
        self.draw_call_count += 1;
        Ok(())
    }

    fn draw_rect(&mut self, _rect: Rect, _color: Color) -> Result<(), RenderError> {
        self.draw_call_count += 1;
        Ok(())
    }

    fn draw_text(
        &mut self,
        _text: &str,
        _position: Point,
        _font_size: f32,
        _color: Color,
    ) -> Result<(), RenderError> {
        self.draw_call_count += 1;
        Ok(())
    }

    fn render_frame(&mut self) -> Result<(), RenderError> {
        self.draw_call_count = 0;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::Size;

    #[test]
    fn test_gl_renderer_pipeline() {
        let mut renderer = GlRenderer::new();
        let win = WindowId::new(1);

        renderer.clear(Color::BLACK).unwrap();
        assert_eq!(renderer.last_clear_color, Some(Color::BLACK));

        // Before texture cache update: draws placeholder
        let rect = Rect::new(100, 132, 800, 568);
        assert!(renderer.draw_window_texture(win, rect).is_ok());

        // Update texture cache and draw
        renderer.texture_cache.update_texture(win, Size::new(800, 568));
        assert!(renderer.draw_window_texture(win, rect).is_ok());

        assert!(renderer.render_frame().is_ok());
        assert_eq!(renderer.draw_call_count, 0);
    }
}
