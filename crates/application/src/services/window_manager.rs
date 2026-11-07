use std::collections::HashMap;

use domain::{
    MruFocusHistory, PlacementPolicy, Point, Rect, ResizeDirection, Size, SnapPosition, Window,
    WindowId, WindowState,
};

use crate::events::DesktopEnvironmentEvent;

/// Mode of an active interactive pointer grab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrabMode {
    Move,
    Resize(ResizeDirection),
}

/// Details of an in-progress interactive move or resize gesture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowGrab {
    pub window_id: WindowId,
    pub mode: GrabMode,
    pub initial_geometry: Rect,
    pub grab_point: Point,
}

/// Advanced Window Manager coordinating floating geometries, z-order stacking,
/// window snapping, interactive move/resize gestures, and MRU focus history.
#[derive(Debug, Clone)]
pub struct WindowManager {
    windows: HashMap<WindowId, Window>,
    stacking_order: Vec<WindowId>,
    focus_history: MruFocusHistory,
    placement_policy: PlacementPolicy,
    screen_rect: Rect,
    work_area: Rect,
    last_window_rect: Option<Rect>,
    pub active_grab: Option<WindowGrab>,
}

impl WindowManager {
    pub fn new(screen_rect: Rect) -> Self {
        Self {
            windows: HashMap::new(),
            stacking_order: Vec::new(),
            focus_history: MruFocusHistory::new(),
            placement_policy: PlacementPolicy::default(),
            screen_rect,
            work_area: screen_rect,
            last_window_rect: None,
            active_grab: None,
        }
    }

    pub fn set_screen_rect(&mut self, rect: Rect) {
        self.screen_rect = rect;
        if self.work_area == self.screen_rect {
            self.work_area = rect;
        }
    }

    pub fn set_work_area(&mut self, area: Rect) {
        self.work_area = area;
    }

    pub fn work_area(&self) -> Rect {
        self.work_area
    }

    pub fn screen_rect(&self) -> Rect {
        self.screen_rect
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
        let geometry =
            self.placement_policy
                .place(self.work_area, initial_size, self.last_window_rect);
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

        // Place on top of stacking order
        self.stacking_order.retain(|&w| w != id);
        self.stacking_order.push(id);

        DesktopEnvironmentEvent::WindowCreated { window }
    }

    pub fn close_window(
        &mut self,
        id: WindowId,
    ) -> (Option<DesktopEnvironmentEvent>, Option<WindowId>) {
        if let Some(ref grab) = self.active_grab {
            if grab.window_id == id {
                self.active_grab = None;
            }
        }

        self.stacking_order.retain(|&w| w != id);

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

        // Raise to top of z-order stack
        self.stacking_order.retain(|&w| w != id);
        self.stacking_order.push(id);

        if let Some(win) = self.windows.get_mut(&id) {
            win.focus();
            self.focus_history.record_focus(id);
            Some(DesktopEnvironmentEvent::WindowFocused { id })
        } else {
            None
        }
    }

    pub fn raise_window(&mut self, id: WindowId) -> Option<DesktopEnvironmentEvent> {
        if !self.windows.contains_key(&id) {
            return None;
        }
        self.stacking_order.retain(|&w| w != id);
        self.stacking_order.push(id);
        Some(DesktopEnvironmentEvent::WindowRaised { id })
    }

    pub fn lower_window(&mut self, id: WindowId) -> Option<DesktopEnvironmentEvent> {
        if !self.windows.contains_key(&id) {
            return None;
        }
        self.stacking_order.retain(|&w| w != id);
        self.stacking_order.insert(0, id);
        Some(DesktopEnvironmentEvent::WindowLowered { id })
    }

    pub fn toggle_maximize(&mut self, id: WindowId) -> Option<DesktopEnvironmentEvent> {
        let work_area = self.work_area;
        let win = self.windows.get_mut(&id)?;
        win.maximize(work_area);
        let state = win.state.clone();
        Some(DesktopEnvironmentEvent::WindowStateChanged { id, state })
    }

    pub fn snap_window(
        &mut self,
        id: WindowId,
        position: SnapPosition,
    ) -> Option<DesktopEnvironmentEvent> {
        let work_area = self.work_area;
        let win = self.windows.get_mut(&id)?;
        win.snap(position, work_area);
        Some(DesktopEnvironmentEvent::WindowSnapped { id, position })
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

    // --- Interactive Grab State Machine (Move / Resize) ---

    pub fn start_move(&mut self, id: WindowId, grab_point: Point) -> bool {
        if let Some(win) = self.windows.get(&id) {
            if win.is_maximized() || win.is_fullscreen() {
                return false;
            }
            self.focus_window(id);
            self.active_grab = Some(WindowGrab {
                window_id: id,
                mode: GrabMode::Move,
                initial_geometry: win.geometry,
                grab_point,
            });
            true
        } else {
            false
        }
    }

    pub fn start_resize(
        &mut self,
        id: WindowId,
        direction: ResizeDirection,
        grab_point: Point,
    ) -> bool {
        if let Some(win) = self.windows.get(&id) {
            if win.is_maximized() || win.is_fullscreen() {
                return false;
            }
            self.focus_window(id);
            self.active_grab = Some(WindowGrab {
                window_id: id,
                mode: GrabMode::Resize(direction),
                initial_geometry: win.geometry,
                grab_point,
            });
            true
        } else {
            false
        }
    }

    pub fn update_grab(
        &mut self,
        current_pointer: Point,
        min_size: Size,
    ) -> Option<DesktopEnvironmentEvent> {
        let grab = self.active_grab.as_ref()?;
        let id = grab.window_id;
        let dx = current_pointer.x - grab.grab_point.x;
        let dy = current_pointer.y - grab.grab_point.y;

        match grab.mode {
            GrabMode::Move => {
                let new_origin = grab.initial_geometry.origin.translated(dx, dy);
                if let Some(win) = self.windows.get_mut(&id) {
                    win.geometry.origin = new_origin;
                    Some(DesktopEnvironmentEvent::WindowMoved {
                        id,
                        new_position: new_origin,
                    })
                } else {
                    None
                }
            }
            GrabMode::Resize(direction) => {
                let new_geo = direction.apply_resize(grab.initial_geometry, dx, dy, min_size);
                if let Some(win) = self.windows.get_mut(&id) {
                    win.geometry = new_geo;
                    Some(DesktopEnvironmentEvent::WindowResized {
                        id,
                        new_size: new_geo.size,
                    })
                } else {
                    None
                }
            }
        }
    }

    pub fn end_grab(&mut self) -> Option<WindowGrab> {
        self.active_grab.take()
    }

    pub fn cancel_grab(&mut self) -> Option<DesktopEnvironmentEvent> {
        let grab = self.active_grab.take()?;
        let id = grab.window_id;
        if let Some(win) = self.windows.get_mut(&id) {
            win.geometry = grab.initial_geometry;
            Some(DesktopEnvironmentEvent::WindowMoved {
                id,
                new_position: grab.initial_geometry.origin,
            })
        } else {
            None
        }
    }

    // --- Queries & Accessors ---

    pub fn get_window(&self, id: WindowId) -> Option<&Window> {
        self.windows.get(&id)
    }

    pub fn get_window_mut(&mut self, id: WindowId) -> Option<&mut Window> {
        self.windows.get_mut(&id)
    }

    /// Returns windows in an unordered iterator.
    pub fn windows(&self) -> impl Iterator<Item = &Window> {
        self.windows.values()
    }

    /// Returns windows in strict z-order stacking (bottom-most to topmost).
    pub fn windows_stacked(&self) -> impl Iterator<Item = &Window> {
        self.stacking_order
            .iter()
            .filter_map(|id| self.windows.get(id))
    }

    /// Returns the topmost window in the z-order stack.
    pub fn top_window(&self) -> Option<&Window> {
        self.stacking_order
            .last()
            .and_then(|id| self.windows.get(id))
    }

    /// Hit-tests for the topmost visible window under a point.
    pub fn window_at(&self, point: Point) -> Option<&Window> {
        for id in self.stacking_order.iter().rev() {
            if let Some(win) = self.windows.get(id) {
                if win.visible && !win.is_minimized() && win.geometry.contains_point(point) {
                    return Some(win);
                }
            }
        }
        None
    }

    pub fn focused_window_id(&self) -> Option<WindowId> {
        self.focus_history.current_focused()
    }

    pub fn focused_window(&self) -> Option<&Window> {
        self.focused_window_id()
            .and_then(|id| self.windows.get(&id))
    }

    pub fn count(&self) -> usize {
        self.windows.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_manager_stacking_order() {
        let mut wm = WindowManager::new(Rect::new(0, 0, 1920, 1080));
        let w1 = WindowId::new(1);
        let w2 = WindowId::new(2);
        let w3 = WindowId::new(3);

        wm.open_window(w1, "W1".into(), None, Size::new(400, 300));
        wm.open_window(w2, "W2".into(), None, Size::new(400, 300));
        wm.open_window(w3, "W3".into(), None, Size::new(400, 300));

        let order: Vec<WindowId> = wm.windows_stacked().map(|w| w.id).collect();
        assert_eq!(order, vec![w1, w2, w3]);
        assert_eq!(wm.top_window().unwrap().id, w3);

        // Focus W1 -> raises W1 to the top
        wm.focus_window(w1);
        let order2: Vec<WindowId> = wm.windows_stacked().map(|w| w.id).collect();
        assert_eq!(order2, vec![w2, w3, w1]);
        assert_eq!(wm.top_window().unwrap().id, w1);

        // Lower W1 -> sends W1 to the bottom
        wm.lower_window(w1);
        let order3: Vec<WindowId> = wm.windows_stacked().map(|w| w.id).collect();
        assert_eq!(order3, vec![w1, w2, w3]);
    }

    #[test]
    fn test_window_snapping() {
        let mut wm = WindowManager::new(Rect::new(0, 0, 1920, 1080));
        wm.set_work_area(Rect::new(0, 30, 1920, 1050));
        let w1 = WindowId::new(1);

        wm.open_window(w1, "Editor".into(), None, Size::new(800, 600));

        // Snap left
        wm.snap_window(w1, SnapPosition::LeftHalf);
        let win = wm.get_window(w1).unwrap();
        assert!(win.is_snapped());
        assert_eq!(win.geometry, Rect::new(0, 30, 960, 1050));

        // Snap right
        wm.snap_window(w1, SnapPosition::RightHalf);
        let win = wm.get_window(w1).unwrap();
        assert_eq!(win.geometry, Rect::new(960, 30, 960, 1050));

        // Restore
        wm.restore_window(w1);
        let win = wm.get_window(w1).unwrap();
        assert!(!win.is_snapped());
        assert_eq!(win.geometry.size, Size::new(800, 600));
    }

    #[test]
    fn test_interactive_grab_move() {
        let mut wm = WindowManager::new(Rect::new(0, 0, 1920, 1080));
        let w1 = WindowId::new(1);
        wm.open_window(w1, "Term".into(), None, Size::new(600, 400));
        let orig = wm.get_window(w1).unwrap().geometry.origin;

        // Start move at (100, 100)
        assert!(wm.start_move(w1, Point::new(100, 100)));
        assert!(wm.active_grab.is_some());

        // Move pointer by (+50, +30)
        wm.update_grab(Point::new(150, 130), Size::new(200, 100));
        let cur = wm.get_window(w1).unwrap().geometry.origin;
        assert_eq!(cur.x, orig.x + 50);
        assert_eq!(cur.y, orig.y + 30);

        // Finish move
        assert!(wm.end_grab().is_some());
        assert!(wm.active_grab.is_none());
    }

    #[test]
    fn test_window_hit_test_topmost() {
        let mut wm = WindowManager::new(Rect::new(0, 0, 1920, 1080));
        let w1 = WindowId::new(1);
        let w2 = WindowId::new(2);

        wm.open_window(w1, "Bottom".into(), None, Size::new(500, 500));
        wm.open_window(w2, "Top".into(), None, Size::new(500, 500));

        // Move both so they overlap at (100, 100)
        wm.set_window_position(w1, Point::new(50, 50));
        wm.set_window_position(w2, Point::new(80, 80));

        // Point (120, 120) is inside both windows; should hit topmost (w2)
        let hit = wm.window_at(Point::new(120, 120));
        assert_eq!(hit.unwrap().id, w2);

        // Raise w1 -> now w1 is topmost
        wm.raise_window(w1);
        let hit2 = wm.window_at(Point::new(120, 120));
        assert_eq!(hit2.unwrap().id, w1);
    }
}
