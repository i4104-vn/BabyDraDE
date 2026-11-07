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
        Self::render_with_overlay(
            renderer,
            screen_rect,
            background,
            windows_in_z_order,
            None,
            cursor,
        )
    }

    pub fn render_with_overlay<'a>(
        renderer: &mut dyn CanvasRenderer,
        screen_rect: Rect,
        background: &CanvasBackground,
        windows_in_z_order: impl Iterator<Item = WindowSceneNode<'a>>,
        launcher: Option<&crate::launcher::LauncherOverlay>,
        cursor: Option<&Cursor>,
    ) -> Result<(), RenderError> {
        Self::render_layers(
            renderer,
            screen_rect,
            background,
            windows_in_z_order,
            launcher,
            cursor,
        )?;
        renderer.render_frame()?;
        Ok(())
    }

    /// Renders scene graph layers (Background -> Windows -> Overlay -> Cursor) into the canvas buffer
    /// without flushing to physical display hardware.
    pub fn render_layers<'a>(
        renderer: &mut dyn CanvasRenderer,
        screen_rect: Rect,
        background: &CanvasBackground,
        windows_in_z_order: impl Iterator<Item = WindowSceneNode<'a>>,
        launcher: Option<&crate::launcher::LauncherOverlay>,
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

            // Draw Server-Side titlebar action buttons: [_] [□] [×] (planning.md 1.3 & Phase 3)
            if node.titlebar_rect.width() >= 100 {
                renderer.draw_text(
                    "[_] [□] [×]",
                    Point::new(node.titlebar_rect.right() - 88, node.titlebar_rect.y() + 8),
                    14.0,
                    node.titlebar_text_color,
                )?;
            }
        }

        // 3. Layer: Overlay (Launcher, dialogs)
        if let Some(l) = launcher {
            l.render(renderer, screen_rect)?;
        }

        // 4. Layer: Cursor
        if let Some(c) = cursor {
            if c.visible {
                draw_cursor_pointer(renderer, c.position.x, c.position.y)?;
            }
        }

        Ok(())
    }
}

/// Draws an authentic 12x18 OS desktop arrow pointer (white body with black outline)
pub fn draw_cursor_pointer(
    renderer: &mut dyn CanvasRenderer,
    x: i32,
    y: i32,
) -> Result<(), RenderError> {
    // 0: transparent, 1: black border, 2: white body
    #[rustfmt::skip]
    const CURSOR_ARROW: [[u8; 12]; 18] = [
        [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [1, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [1, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0],
        [1, 2, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0],
        [1, 2, 2, 2, 2, 1, 0, 0, 0, 0, 0, 0],
        [1, 2, 2, 2, 2, 2, 1, 0, 0, 0, 0, 0],
        [1, 2, 2, 2, 2, 2, 2, 1, 0, 0, 0, 0],
        [1, 2, 2, 2, 2, 2, 2, 2, 1, 0, 0, 0],
        [1, 2, 2, 2, 2, 2, 2, 2, 2, 1, 0, 0],
        [1, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 0],
        [1, 2, 1, 2, 2, 1, 0, 0, 0, 0, 0, 0],
        [1, 1, 0, 1, 2, 2, 1, 0, 0, 0, 0, 0],
        [1, 0, 0, 1, 2, 2, 1, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 1, 2, 2, 1, 0, 0, 0, 0],
        [0, 0, 0, 0, 1, 2, 2, 1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];

    for (row, line) in CURSOR_ARROW.iter().enumerate() {
        for (col, &pixel) in line.iter().enumerate() {
            let px = x + col as i32;
            let py = y + row as i32;
            match pixel {
                1 => renderer.draw_rect(Rect::new(px, py, 1, 1), Color::BLACK)?,
                2 => renderer.draw_rect(Rect::new(px, py, 1, 1), Color::WHITE)?,
                _ => {}
            }
        }
    }

    Ok(())
}
