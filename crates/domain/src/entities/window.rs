use crate::value_objects::Rect;
pub use crate::value_objects::SnapPosition;
use serde::{Deserialize, Serialize};

/// Unique identifier for a managed window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct WindowId(pub u64);

impl WindowId {
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for WindowId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Window({})", self.0)
    }
}

/// State of a floating window.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowState {
    Normal,
    Maximized { saved_geometry: Rect },
    Minimized,
    Fullscreen { saved_geometry: Rect },
    Snapped { position: SnapPosition, saved_geometry: Rect },
}

/// Core Window entity in BabyDraDE.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Window {
    pub id: WindowId,
    pub title: String,
    pub app_id: Option<String>,
    pub geometry: Rect,
    pub state: WindowState,
    pub focused: bool,
    pub decorated: bool,
    pub visible: bool,
}

impl Window {
    pub fn new(id: WindowId, title: impl Into<String>, geometry: Rect) -> Self {
        Self {
            id,
            title: title.into(),
            app_id: None,
            geometry,
            state: WindowState::Normal,
            focused: false,
            decorated: true,
            visible: true,
        }
    }

    pub fn with_app_id(mut self, app_id: impl Into<String>) -> Self {
        self.app_id = Some(app_id.into());
        self
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
    }

    pub fn set_app_id(&mut self, app_id: Option<String>) {
        self.app_id = app_id;
    }

    pub fn set_geometry(&mut self, geometry: Rect) {
        self.geometry = geometry;
    }

    pub fn maximize(&mut self, screen_rect: Rect) {
        match self.state {
            WindowState::Normal => {
                let saved = self.geometry;
                self.state = WindowState::Maximized {
                    saved_geometry: saved,
                };
                self.geometry = screen_rect;
                self.visible = true;
            }
            WindowState::Maximized { .. } => {
                self.restore();
            }
            _ => {
                self.state = WindowState::Maximized {
                    saved_geometry: self.geometry,
                };
                self.geometry = screen_rect;
                self.visible = true;
            }
        }
    }

    pub fn snap(&mut self, position: SnapPosition, work_area: Rect) {
        let saved = match self.state {
            WindowState::Normal => self.geometry,
            WindowState::Maximized { saved_geometry }
            | WindowState::Fullscreen { saved_geometry }
            | WindowState::Snapped { saved_geometry, .. } => saved_geometry,
            WindowState::Minimized => self.geometry,
        };

        let half_w = work_area.width() / 2;
        let half_h = work_area.height() / 2;

        let target_rect = match position {
            SnapPosition::LeftHalf => Rect::new(work_area.x(), work_area.y(), half_w, work_area.height()),
            SnapPosition::RightHalf => {
                Rect::new(work_area.x() + half_w as i32, work_area.y(), half_w, work_area.height())
            }
            SnapPosition::TopHalf => Rect::new(work_area.x(), work_area.y(), work_area.width(), half_h),
            SnapPosition::BottomHalf => {
                Rect::new(work_area.x(), work_area.y() + half_h as i32, work_area.width(), half_h)
            }
        };

        self.geometry = target_rect;
        self.state = WindowState::Snapped {
            position,
            saved_geometry: saved,
        };
        self.visible = true;
    }

    pub fn restore(&mut self) {
        match self.state {
            WindowState::Maximized { saved_geometry }
            | WindowState::Fullscreen { saved_geometry }
            | WindowState::Snapped { saved_geometry, .. } => {
                self.geometry = saved_geometry;
                self.state = WindowState::Normal;
                self.visible = true;
            }
            WindowState::Minimized => {
                self.state = WindowState::Normal;
                self.visible = true;
            }
            WindowState::Normal => {}
        }
    }

    pub fn minimize(&mut self) {
        self.state = WindowState::Minimized;
        self.visible = false;
        self.focused = false;
    }

    pub fn fullscreen(&mut self, screen_rect: Rect) {
        match self.state {
            WindowState::Fullscreen { .. } => {
                self.restore();
            }
            WindowState::Normal
            | WindowState::Maximized { .. }
            | WindowState::Snapped { .. } => {
                let saved = match self.state {
                    WindowState::Maximized { saved_geometry }
                    | WindowState::Snapped { saved_geometry, .. } => saved_geometry,
                    _ => self.geometry,
                };
                self.state = WindowState::Fullscreen {
                    saved_geometry: saved,
                };
                self.geometry = screen_rect;
                self.visible = true;
            }
            WindowState::Minimized => {
                self.state = WindowState::Fullscreen {
                    saved_geometry: self.geometry,
                };
                self.geometry = screen_rect;
                self.visible = true;
            }
        }
    }

    pub fn focus(&mut self) {
        self.focused = true;
        if self.state == WindowState::Minimized {
            self.restore();
        }
    }

    pub fn unfocus(&mut self) {
        self.focused = false;
    }

    pub fn is_maximized(&self) -> bool {
        matches!(self.state, WindowState::Maximized { .. })
    }

    pub fn is_minimized(&self) -> bool {
        matches!(self.state, WindowState::Minimized)
    }

    pub fn is_fullscreen(&self) -> bool {
        matches!(self.state, WindowState::Fullscreen { .. })
    }

    pub fn is_snapped(&self) -> bool {
        matches!(self.state, WindowState::Snapped { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_lifecycle() {
        let mut win = Window::new(WindowId::new(1), "Alacritty", Rect::new(100, 100, 800, 600));
        assert_eq!(win.title, "Alacritty");
        assert!(!win.focused);
        assert!(win.visible);

        win.focus();
        assert!(win.focused);

        let screen = Rect::new(0, 0, 1920, 1080);
        win.maximize(screen);
        assert!(win.is_maximized());
        assert_eq!(win.geometry, screen);

        // Maximize toggle restores
        win.maximize(screen);
        assert_eq!(win.geometry, Rect::new(100, 100, 800, 600));
        assert_eq!(win.state, WindowState::Normal);

        win.minimize();
        assert!(win.is_minimized());
        assert!(!win.visible);
        assert!(!win.focused);

        win.restore();
        assert_eq!(win.state, WindowState::Normal);
        assert!(win.visible);
    }
}
