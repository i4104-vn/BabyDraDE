use domain::{CanvasRenderer, Color, Cursor, Point, Rect, RenderError, Window};

use crate::canvas::CanvasBackground;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LayerZIndex {
    Background = 0,
    Windows = 1,
    Overlay = 2,
    Cursor = 3,
}

/// Node representing a window to be drawn in the scene graph.
pub struct WindowSceneNode<'a> {
    pub window: &'a Window,
    pub titlebar_rect: Rect,
    pub client_rect: Rect,
    pub border_rects: [Rect; 4],
    pub border_color: Color,
    pub titlebar_bg_color: Color,
    pub titlebar_text_color: Color,
}

/// Orchestrates scene rendering in strict layer order:
/// Background -> Windows (z-order lowest to highest) -> Overlay -> Cursor
pub struct SceneGraph;

impl SceneGraph {
    pub fn render<'a>(
        renderer: &mut dyn CanvasRenderer,
        screen_rect: Rect,
        background: &CanvasBackground,
        windows_in_z_order: impl Iterator<Item = WindowSceneNode<'a>>,
        cursor: Option<&Cursor>,
    ) -> Result<(), RenderError> {
        // 1. Layer: Background
        background.render(renderer, screen_rect)?;

        // 2. Layer: Windows
        for node in windows_in_z_order {
            if !node.window.visible || node.window.is_minimized() {
                continue;
            }

            // Draw window background / client area placeholder
            renderer.draw_rect(node.client_rect, Color::rgb(20, 20, 25))?;

            // Draw borders
            for rect in &node.border_rects {
                renderer.draw_rect(*rect, node.border_color)?;
            }

            // Draw titlebar
            renderer.draw_rect(node.titlebar_rect, node.titlebar_bg_color)?;
            renderer.draw_text(
                &node.window.title,
                Point::new(node.titlebar_rect.x() + 10, node.titlebar_rect.y() + 8),
                14.0,
                node.titlebar_text_color,
            )?;
        }

        // 3. Layer: Cursor
        if let Some(c) = cursor {
            if c.visible {
                // Draw cursor pointer indicator (12x12 white square with black border for dev)
                let cur_rect = Rect::new(c.position.x, c.position.y, 10, 10);
                renderer.draw_rect(cur_rect, Color::WHITE)?;
            }
        }

        renderer.render_frame()?;
        Ok(())
    }
}
