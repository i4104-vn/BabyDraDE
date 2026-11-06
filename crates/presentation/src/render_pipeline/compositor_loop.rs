use application::WindowManager;
use domain::{CanvasRenderer, Color, Cursor, Rect, RenderError, WindowId};

use crate::canvas::CanvasBackground;
use crate::render_pipeline::scene_graph::{SceneGraph, WindowSceneNode};
use crate::window_frame::{BorderDecorator, TitlebarDecorator};

/// Main presentation loop coordinator.
pub struct CompositorRenderPipeline {
    pub background: CanvasBackground,
    pub titlebar_decorator: TitlebarDecorator,
    pub border_decorator: BorderDecorator,
}

impl CompositorRenderPipeline {
    pub fn new() -> Self {
        Self {
            background: CanvasBackground::default(),
            titlebar_decorator: TitlebarDecorator::default(),
            border_decorator: BorderDecorator::default(),
        }
    }

    /// Renders frame with optional workspace window filtering, launcher overlay and fullscreen handling.
    pub fn render_frame_full(
        &self,
        renderer: &mut dyn CanvasRenderer,
        window_manager: &WindowManager,
        active_window_ids: Option<&[WindowId]>,
        launcher: Option<&crate::launcher::LauncherOverlay>,
        screen_rect: Rect,
        cursor: Option<&Cursor>,
    ) -> Result<(), RenderError> {
        let nodes = window_manager
            .windows()
            .filter(|win| {
                if !win.visible || win.is_minimized() {
                    return false;
                }
                if let Some(ids) = active_window_ids {
                    ids.contains(&win.id)
                } else {
                    true
                }
            })
            .map(|win| {
                if win.is_fullscreen() {
                    WindowSceneNode {
                        window: win,
                        titlebar_rect: Rect::ZERO,
                        client_rect: screen_rect,
                        border_rects: [Rect::ZERO; 4],
                        border_color: Color::TRANSPARENT,
                        titlebar_bg_color: Color::TRANSPARENT,
                        titlebar_text_color: Color::TRANSPARENT,
                    }
                } else {
                    let titlebar_rect = self.titlebar_decorator.titlebar_rect(&win.geometry);
                    let client_rect = self.titlebar_decorator.client_rect(&win.geometry);
                    let border_rects = self.border_decorator.border_rects(&win.geometry);
                    let border_color = self.border_decorator.border_color(win.focused);

                    let (titlebar_bg, titlebar_text) = if win.focused {
                        (
                            self.titlebar_decorator.theme.background_focused,
                            self.titlebar_decorator.theme.text_color_focused,
                        )
                    } else {
                        (
                            self.titlebar_decorator.theme.background_unfocused,
                            self.titlebar_decorator.theme.text_color_unfocused,
                        )
                    };

                    WindowSceneNode {
                        window: win,
                        titlebar_rect,
                        client_rect,
                        border_rects,
                        border_color,
                        titlebar_bg_color: titlebar_bg,
                        titlebar_text_color: titlebar_text,
                    }
                }
            });

        SceneGraph::render_with_overlay(
            renderer,
            screen_rect,
            &self.background,
            nodes,
            launcher,
            cursor,
        )
    }

    pub fn render_frame_with_workspace(
        &self,
        renderer: &mut dyn CanvasRenderer,
        window_manager: &WindowManager,
        active_window_ids: Option<&[WindowId]>,
        screen_rect: Rect,
        cursor: Option<&Cursor>,
    ) -> Result<(), RenderError> {
        self.render_frame_full(
            renderer,
            window_manager,
            active_window_ids,
            None,
            screen_rect,
            cursor,
        )
    }

    pub fn render_frame(
        &self,
        renderer: &mut dyn CanvasRenderer,
        window_manager: &WindowManager,
        screen_rect: Rect,
        cursor: Option<&Cursor>,
    ) -> Result<(), RenderError> {
        self.render_frame_with_workspace(renderer, window_manager, None, screen_rect, cursor)
    }
}

impl Default for CompositorRenderPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::{Color, Point, Size, WindowId};

    // Mock renderer for testing pipeline execution
    #[derive(Default)]
    struct MockRenderer {
        cleared: bool,
        drawn_rects: Vec<(Rect, Color)>,
        frame_rendered: bool,
    }

    impl CanvasRenderer for MockRenderer {
        fn clear(&mut self, _color: Color) -> Result<(), RenderError> {
            self.cleared = true;
            Ok(())
        }
        fn draw_rect(&mut self, rect: Rect, color: Color) -> Result<(), RenderError> {
            self.drawn_rects.push((rect, color));
            Ok(())
        }
        fn draw_text(
            &mut self,
            _text: &str,
            _position: Point,
            _size: f32,
            _color: Color,
        ) -> Result<(), RenderError> {
            Ok(())
        }
        fn render_frame(&mut self) -> Result<(), RenderError> {
            self.frame_rendered = true;
            Ok(())
        }
    }

    #[test]
    fn test_render_pipeline_execution() {
        let pipeline = CompositorRenderPipeline::default();
        let screen = Rect::new(0, 0, 1920, 1080);
        let mut wm = WindowManager::new(screen);
        wm.open_window(
            WindowId::new(1),
            "TestApp".into(),
            None,
            Size::new(800, 600),
        );

        let mut renderer = MockRenderer::default();
        let cursor = Cursor::new(Point::new(100, 100));

        let res = pipeline.render_frame(&mut renderer, &wm, screen, Some(&cursor));
        assert!(res.is_ok());
        assert!(renderer.cleared);
        assert!(!renderer.drawn_rects.is_empty());
        assert!(renderer.frame_rendered);
    }

    #[test]
    fn test_render_pipeline_workspace_filtering() {
        let pipeline = CompositorRenderPipeline::default();
        let screen = Rect::new(0, 0, 1920, 1080);
        let mut wm = WindowManager::new(screen);

        let w1 = WindowId::new(1);
        let w2 = WindowId::new(2);
        wm.open_window(w1, "App1".into(), None, Size::new(400, 300));
        wm.open_window(w2, "App2".into(), None, Size::new(400, 300));

        let mut renderer = MockRenderer::default();

        // Only render workspace with window 1
        let active_ids = vec![w1];
        let res = pipeline.render_frame_with_workspace(
            &mut renderer,
            &wm,
            Some(&active_ids),
            screen,
            None,
        );
        assert!(res.is_ok());
    }
}
