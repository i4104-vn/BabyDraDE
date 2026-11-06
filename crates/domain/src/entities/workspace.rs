use super::window::WindowId;
use serde::{Deserialize, Serialize};

/// Unique identifier for a workspace (1-based index typically).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct WorkspaceId(pub u32);

impl WorkspaceId {
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for WorkspaceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Workspace({})", self.0)
    }
}

/// Workspace entity grouping a collection of windows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Workspace {
    pub id: WorkspaceId,
    pub name: String,
    pub windows: Vec<WindowId>,
    pub active_window: Option<WindowId>,
}

impl Workspace {
    pub fn new(id: WorkspaceId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            windows: Vec::new(),
            active_window: None,
        }
    }

    pub fn add_window(&mut self, id: WindowId) {
        if !self.contains_window(id) {
            self.windows.push(id);
        }
        self.active_window = Some(id);
    }

    pub fn remove_window(&mut self, id: WindowId) -> bool {
        let original_len = self.windows.len();
        self.windows.retain(|&w| w != id);
        let removed = self.windows.len() < original_len;

        if removed && self.active_window == Some(id) {
            self.active_window = self.windows.last().copied();
        }

        removed
    }

    pub fn contains_window(&self, id: WindowId) -> bool {
        self.windows.contains(&id)
    }

    pub fn raise_window(&mut self, id: WindowId) {
        if let Some(pos) = self.windows.iter().position(|&w| w == id) {
            self.windows.remove(pos);
            self.windows.push(id);
            self.active_window = Some(id);
        }
    }

    pub fn set_active_window(&mut self, id: Option<WindowId>) {
        if let Some(win_id) = id {
            if self.contains_window(win_id) {
                self.raise_window(win_id);
            }
        } else {
            self.active_window = None;
        }
    }

    pub fn windows(&self) -> &[WindowId] {
        &self.windows
    }

    pub fn is_empty(&self) -> bool {
        self.windows.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_operations() {
        let mut ws = Workspace::new(WorkspaceId::new(1), "Main");
        let w1 = WindowId::new(1);
        let w2 = WindowId::new(2);

        ws.add_window(w1);
        assert_eq!(ws.active_window, Some(w1));

        ws.add_window(w2);
        assert_eq!(ws.active_window, Some(w2));
        assert_eq!(ws.windows(), &[w1, w2]);

        ws.raise_window(w1);
        assert_eq!(ws.windows(), &[w2, w1]);
        assert_eq!(ws.active_window, Some(w1));

        ws.remove_window(w1);
        assert_eq!(ws.windows(), &[w2]);
        assert_eq!(ws.active_window, Some(w2));
    }
}
