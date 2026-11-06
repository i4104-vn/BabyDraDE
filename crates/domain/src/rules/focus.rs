use crate::entities::WindowId;
use serde::{Deserialize, Serialize};

/// Most Recently Used (MRU) focus tracking and policy.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct MruFocusHistory {
    /// Front of vector is the most recently focused window.
    stack: Vec<WindowId>,
}

impl MruFocusHistory {
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    /// Records that a window gained focus, promoting it to the front of MRU.
    pub fn record_focus(&mut self, id: WindowId) {
        if let Some(pos) = self.stack.iter().position(|&w| w == id) {
            self.stack.remove(pos);
        }
        self.stack.insert(0, id);
    }

    /// Removes a window from history (e.g. when closed) and returns the next window to focus (MRU head).
    pub fn remove_window(&mut self, id: WindowId) -> Option<WindowId> {
        if let Some(pos) = self.stack.iter().position(|&w| w == id) {
            self.stack.remove(pos);
        }
        self.stack.first().copied()
    }

    /// Returns the currently focused window according to MRU.
    pub fn current_focused(&self) -> Option<WindowId> {
        self.stack.first().copied()
    }

    /// Cycles focus through windows (e.g., for Alt+Tab / Super+Tab).
    /// `forward = true` selects next window in history, `forward = false` selects previous/last.
    pub fn cycle_focus(&mut self, forward: bool) -> Option<WindowId> {
        if self.stack.len() <= 1 {
            return self.stack.first().copied();
        }

        if forward {
            let front = self.stack.remove(0);
            self.stack.push(front);
        } else {
            let back = self.stack.pop().unwrap();
            self.stack.insert(0, back);
        }

        self.stack.first().copied()
    }

    /// Returns the windows in MRU order (most recent first).
    pub fn as_slice(&self) -> &[WindowId] {
        &self.stack
    }

    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    pub fn len(&self) -> usize {
        self.stack.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mru_focus_ordering() {
        let mut history = MruFocusHistory::new();
        let w1 = WindowId::new(1);
        let w2 = WindowId::new(2);
        let w3 = WindowId::new(3);

        history.record_focus(w1);
        history.record_focus(w2);
        history.record_focus(w3);
        assert_eq!(history.current_focused(), Some(w3));
        assert_eq!(history.as_slice(), &[w3, w2, w1]);

        // Re-focus w1 -> moves to front
        history.record_focus(w1);
        assert_eq!(history.current_focused(), Some(w1));
        assert_eq!(history.as_slice(), &[w1, w3, w2]);
    }

    #[test]
    fn test_mru_close_window_fallback() {
        let mut history = MruFocusHistory::new();
        let w1 = WindowId::new(1);
        let w2 = WindowId::new(2);
        let w3 = WindowId::new(3);

        history.record_focus(w1);
        history.record_focus(w2);
        history.record_focus(w3);

        // Close w3 -> next focus falls back to w2
        let next = history.remove_window(w3);
        assert_eq!(next, Some(w2));
        assert_eq!(history.current_focused(), Some(w2));

        // Close w2 -> next focus falls back to w1
        let next = history.remove_window(w2);
        assert_eq!(next, Some(w1));

        // Close w1 -> no windows left
        let next = history.remove_window(w1);
        assert_eq!(next, None);
        assert!(history.is_empty());
    }

    #[test]
    fn test_cycle_focus() {
        let mut history = MruFocusHistory::new();
        let w1 = WindowId::new(1);
        let w2 = WindowId::new(2);
        let w3 = WindowId::new(3);

        history.record_focus(w1);
        history.record_focus(w2);
        history.record_focus(w3); // [w3, w2, w1]

        let next = history.cycle_focus(true);
        assert_eq!(next, Some(w2)); // [w2, w1, w3]

        let next = history.cycle_focus(true);
        assert_eq!(next, Some(w1)); // [w1, w3, w2]

        let prev = history.cycle_focus(false);
        assert_eq!(prev, Some(w2)); // [w2, w1, w3]
    }
}
