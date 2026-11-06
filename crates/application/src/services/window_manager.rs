use std::collections::HashMap;

use domain::{
    MruFocusHistory, PlacementPolicy, Point, Rect, Size, Window, WindowId, WindowState,
};

use crate::events::DesktopEnvironmentEvent;

/// Manages the collection of windows, geometries, focus and placement rules.
#[derive(Debug, Clone)]
pub struct WindowManager {
    windows: HashMap<WindowId, Window>,
    focus_history: MruFocusHistory,
    placement_policy: PlacementPolicy,
    screen_rect: Rect,
    last_window_rect: Option<Rect>,
}

impl WindowManager {
    pub fn new(screen_rect: Rect) -> Self {
        Self {
            windows: HashMap::new(),
            focus_history: MruFocusHistory::new(),
            placement_policy: PlacementPolicy::default(),
            screen_rect,
            last_window_rect: None,
        }
    }

    pub fn set_screen_rect(&mut self, rect: Rect) {
        self.screen_rect = rect;
    }

    pub fn set_placement_policy(&mut self, policy: PlacementPolicy) {
        self.placement_policy = policy;
    }

    pub fn open_window(
        &mut self,
        id: WindowId,
        title: String,
        app_id: Option<String>,
        initial_size: Size,
    ) -> DesktopEnvironmentEvent {
        let geometry = self
            .placement_policy
            .place(self.screen_rect, initial_size, self.last_window_rect);
        self.last_window_rect = Some(geometry);

        let mut window = Window::new(id, title, geometry);
        window.app_id = app_id;

        // Unfocus previous
        if let Some(prev_id) = self.focus_history.current_focused() {
            if let Some(prev) = self.windows.get_mut(&prev_id) {
                prev.unfocus();
            }
        }

        window.focus();
        self.focus_history.record_focus(id);
        self.windows.insert(id, window.clone());

        DesktopEnvironmentEvent::WindowCreated { window }
    }

    pub fn close_window(
        &mut self,
        id: WindowId,
    ) -> (Option<DesktopEnvironmentEvent>, Option<WindowId>) {
        if self.windows.remove(&id).is_some() {
            let next_focus = self.focus_history.remove_window(id);
            if let Some(next_id) = next_focus {
                if let Some(win) = self.windows.get_mut(&next_id) {
                    win.focus();
                }
            }
            (
                Some(DesktopEnvironmentEvent::WindowClosed { id }),
                next_focus,
            )
        } else {
            (None, None)
        }
    }

    pub fn move_window(
        &mut self,
        id: WindowId,
        dx: i32,
        dy: i32,
    ) -> Option<DesktopEnvironmentEvent> {
        let win = self.windows.get_mut(&id)?;
        if win.is_maximized() || win.is_fullscreen() {
            return None;
        }

        win.geometry = win.geometry.translated(dx, dy);
        let new_pos = win.geometry.origin;
        Some(DesktopEnvironmentEvent::WindowMoved {
            id,
            new_position: new_pos,
        })
    }

    pub fn set_window_position(
        &mut self,
        id: WindowId,
        pos: Point,
    ) -> Option<DesktopEnvironmentEvent> {
        let win = self.windows.get_mut(&id)?;
        if win.is_maximized() || win.is_fullscreen() {
            return None;
        }

        win.geometry.origin = pos;
        Some(DesktopEnvironmentEvent::WindowMoved {
            id,
            new_position: pos,
        })
    }

    pub fn resize_window(
        &mut self,
        id: WindowId,
        new_size: Size,
    ) -> Option<DesktopEnvironmentEvent> {
        let win = self.windows.get_mut(&id)?;
        win.geometry.size = new_size;
        Some(DesktopEnvironmentEvent::WindowResized { id, new_size })
    }

    pub fn focus_window(&mut self, id: WindowId) -> Option<DesktopEnvironmentEvent> {
        if !self.windows.contains_key(&id) {
            return None;
        }

        if let Some(prev_id) = self.focus_history.current_focused() {
            if prev_id != id {
                if let Some(prev) = self.windows.get_mut(&prev_id) {
                    prev.unfocus();
                }
            }
        }

        if let Some(win) = self.windows.get_mut(&id) {
            win.focus();
            self.focus_history.record_focus(id);
            Some(DesktopEnvironmentEvent::WindowFocused { id })
        } else {
            None
        }
    }

    pub fn toggle_maximize(&mut self, id: WindowId) -> Option<DesktopEnvironmentEvent> {
        let screen = self.screen_rect;
        let win = self.windows.get_mut(&id)?;
        win.maximize(screen);
        let state = win.state.clone();
        Some(DesktopEnvironmentEvent::WindowStateChanged { id, state })
    }

    pub fn minimize_window(
        &mut self,
        id: WindowId,
    ) -> (Option<DesktopEnvironmentEvent>, Option<WindowId>) {
        let win = match self.windows.get_mut(&id) {
            Some(w) => w,
            None => return (None, None),
        };

        win.minimize();
        let next_focus = self.focus_history.remove_window(id);
        if let Some(next_id) = next_focus {
            if let Some(next_win) = self.windows.get_mut(&next_id) {
                next_win.focus();
            }
        }

        (
            Some(DesktopEnvironmentEvent::WindowStateChanged {
                id,
                state: WindowState::Minimized,
            }),
            next_focus,
        )
    }

    pub fn restore_window(&mut self, id: WindowId) -> Option<DesktopEnvironmentEvent> {
        let win = self.windows.get_mut(&id)?;
        win.restore();
        self.focus_history.record_focus(id);
        let state = win.state.clone();
        Some(DesktopEnvironmentEvent::WindowStateChanged { id, state })
    }

    pub fn toggle_fullscreen(&mut self, id: WindowId) -> Option<DesktopEnvironmentEvent> {
        let screen = self.screen_rect;
        let win = self.windows.get_mut(&id)?;
        win.fullscreen(screen);
        let state = win.state.clone();
        Some(DesktopEnvironmentEvent::WindowStateChanged { id, state })
    }

    pub fn cycle_focus(&mut self, forward: bool) -> Option<WindowId> {
        let next_id = self.focus_history.cycle_focus(forward)?;
        self.focus_window(next_id);
        Some(next_id)
    }

    pub fn get_window(&self, id: WindowId) -> Option<&Window> {
        self.windows.get(&id)
    }

    pub fn get_window_mut(&mut self, id: WindowId) -> Option<&mut Window> {
        self.windows.get_mut(&id)
    }

    pub fn windows(&self) -> impl Iterator<Item = &Window> {
        self.windows.values()
    }

    pub fn focused_window_id(&self) -> Option<WindowId> {
        self.focus_history.current_focused()
    }

    pub fn focused_window(&self) -> Option<&Window> {
        self.focused_window_id().and_then(|id| self.windows.get(&id))
    }

    pub fn count(&self) -> usize {
        self.windows.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_manager_open_and_cascade() {
        let mut wm = WindowManager::new(Rect::new(0, 0, 1920, 1080));
        let w1 = WindowId::new(1);
        let w2 = WindowId::new(2);

        let ev1 = wm.open_window(w1, "Terminal".into(), Some("alacritty".into()), Size::new(800, 600));
        assert!(matches!(ev1, DesktopEnvironmentEvent::WindowCreated { .. }));
        assert_eq!(wm.focused_window_id(), Some(w1));

        let ev2 = wm.open_window(w2, "Browser".into(), Some("firefox".into()), Size::new(800, 600));
        assert!(matches!(ev2, DesktopEnvironmentEvent::WindowCreated { .. }));
        assert_eq!(wm.focused_window_id(), Some(w2));

        // Window 1 is unfocused, Window 2 is focused
        assert!(!wm.get_window(w1).unwrap().focused);
        assert!(wm.get_window(w2).unwrap().focused);

        // Cascade offset check
        let geom1 = wm.get_window(w1).unwrap().geometry;
        let geom2 = wm.get_window(w2).unwrap().geometry;
        assert_eq!(geom2.origin.x, geom1.origin.x + 30);
        assert_eq!(geom2.origin.y, geom1.origin.y + 30);
    }

    #[test]
    fn test_window_manager_close_fallback() {
        let mut wm = WindowManager::new(Rect::new(0, 0, 1920, 1080));
        let w1 = WindowId::new(1);
        let w2 = WindowId::new(2);

        wm.open_window(w1, "W1".into(), None, Size::new(600, 400));
        wm.open_window(w2, "W2".into(), None, Size::new(600, 400));
        assert_eq!(wm.focused_window_id(), Some(w2));

        let (_, next_focus) = wm.close_window(w2);
        assert_eq!(next_focus, Some(w1));
        assert_eq!(wm.focused_window_id(), Some(w1));
        assert!(wm.get_window(w1).unwrap().focused);
    }
}
