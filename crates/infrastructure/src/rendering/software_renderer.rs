use domain::{CanvasRenderer, Color, Point, Rect, RenderError};
use std::fs::File;
use std::io::Write;

/// Clean in-memory 2D Canvas Surface Renderer for BabyDraDE.
/// Provides software rasterization for tests, headless verification,
/// and canvas pipeline validation.
pub struct SoftwareRenderer {
    pub width: u32,
    pub height: u32,
    pub buffer: Vec<u32>,
    pub frame_count: u64,
}

impl SoftwareRenderer {
    /// Creates a new Canvas pixel surface with specified dimensions.
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width as usize) * (height as usize);
        Self {
            width,
            height,
            buffer: vec![0xFF1E1E2E; size],
            frame_count: 0,
        }
    }

    /// Detects display resolution from environment or falls back to standard 1080p canvas.
    pub fn auto_detect() -> Self {
        let width = std::env::var("BABYDRADE_WIDTH")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1920);
        let height = std::env::var("BABYDRADE_HEIGHT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1080);

        tracing::info!("Canvas Surface initialized: {}x{}", width, height);
        Self::new(width, height)
    }

    /// Converts domain::Color (RGBA) into packed ARGB 32-bit pixel: 0xAARRGGBB.
    #[inline]
    pub fn pack_color(color: Color) -> u32 {
        ((color.a as u32) << 24)
            | ((color.r as u32) << 16)
            | ((color.g as u32) << 8)
            | (color.b as u32)
    }

    /// Alpha blends a source color onto an existing destination ARGB pixel.
    #[inline]
    fn blend_pixel(src: Color, dst: u32) -> u32 {
        if src.a == 255 {
            return Self::pack_color(src);
        }
        if src.a == 0 {
            return dst;
        }

        let sa = src.a as u32;
        let da = 255 - sa;

        let sr = src.r as u32;
        let sg = src.g as u32;
        let sb = src.b as u32;

        let dr = (dst >> 16) & 0xFF;
        let dg = (dst >> 8) & 0xFF;
        let db = dst & 0xFF;

        let r = (sr * sa + dr * da) / 255;
        let g = (sg * sa + dg * da) / 255;
        let b = (sb * sa + db * da) / 255;

        (0xFF << 24) | (r << 16) | (g << 8) | b
    }

    /// Exports current canvas state to a PPM image file for visual verification.
    pub fn save_ppm(&self, path: &str) -> std::io::Result<()> {
        let mut file = File::create(path)?;
        writeln!(file, "P6\n{} {}\n255", self.width, self.height)?;
        let mut bytes = Vec::with_capacity((self.width * self.height * 3) as usize);
        for &px in &self.buffer {
            bytes.push(((px >> 16) & 0xFF) as u8);
            bytes.push(((px >> 8) & 0xFF) as u8);
            bytes.push((px & 0xFF) as u8);
        }
        file.write_all(&bytes)?;
        Ok(())
    }
}

impl CanvasRenderer for SoftwareRenderer {
    fn clear(&mut self, color: Color) -> Result<(), RenderError> {
        let px = Self::pack_color(color);
        self.buffer.fill(px);
        Ok(())
    }

    fn draw_rect(&mut self, rect: Rect, color: Color) -> Result<(), RenderError> {
        if color.a == 0 || rect.width() == 0 || rect.height() == 0 {
            return Ok(());
        }

        let x0 = rect.x().max(0) as u32;
        let y0 = rect.y().max(0) as u32;
        let x1 = ((rect.x() + rect.width() as i32).max(0) as u32).min(self.width);
        let y1 = ((rect.y() + rect.height() as i32).max(0) as u32).min(self.height);

        if x0 >= x1 || y0 >= y1 {
            return Ok(());
        }

        if color.a == 255 {
            let px = Self::pack_color(color);
            for y in y0..y1 {
                let row_start = (y * self.width + x0) as usize;
                let row_end = (y * self.width + x1) as usize;
                self.buffer[row_start..row_end].fill(px);
            }
        } else {
            for y in y0..y1 {
                let row_offset = (y * self.width) as usize;
                for x in x0..x1 {
                    let idx = row_offset + x as usize;
                    self.buffer[idx] = Self::blend_pixel(color, self.buffer[idx]);
                }
            }
        }

        Ok(())
    }

    fn draw_text(
        &mut self,
        _text: &str,
        _position: Point,
        _font_size: f32,
        _color: Color,
    ) -> Result<(), RenderError> {
        // High-level text rendering is handled by the presentation pipeline / GPU shader
        Ok(())
    }

    fn render_frame(&mut self) -> Result<(), RenderError> {
        self.frame_count += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_renderer_pipeline() {
        let mut r = SoftwareRenderer::new(800, 600);
        assert_eq!(r.width, 800);
        assert_eq!(r.height, 600);

        // Clear canvas
        assert!(r.clear(Color::rgb(20, 20, 30)).is_ok());

        // Draw rect on canvas
        let rect = Rect::new(50, 50, 200, 100);
        assert!(r.draw_rect(rect, Color::rgb(255, 0, 0)).is_ok());

        // Render frame to canvas
        assert!(r.render_frame().is_ok());
        assert_eq!(r.frame_count, 1);
    }
}
