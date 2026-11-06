use std::collections::BTreeMap;

use domain::{WindowId, Workspace, WorkspaceId};

use crate::events::DesktopEnvironmentEvent;

/// Manages multiple virtual workspaces.
#[derive(Debug, Clone)]
pub struct WorkspaceManager {
    workspaces: BTreeMap<WorkspaceId, Workspace>,
    active_workspace: WorkspaceId,
}

impl WorkspaceManager {
    pub fn new(count: u32) -> Self {
        let mut workspaces = BTreeMap::new();
        let default_count = count.max(1);

        for i in 1..=default_count {
            let id = WorkspaceId::new(i);
            workspaces.insert(id, Workspace::new(id, format!("Workspace {i}")));
        }

        Self {
            workspaces,
            active_workspace: WorkspaceId::new(1),
        }
    }

    pub fn active_workspace_id(&self) -> WorkspaceId {
        self.active_workspace
    }

    pub fn current_workspace(&self) -> &Workspace {
        self.workspaces
            .get(&self.active_workspace)
            .expect("Active workspace must exist")
    }

    pub fn current_workspace_mut(&mut self) -> &mut Workspace {
        self.workspaces
            .get_mut(&self.active_workspace)
            .expect("Active workspace must exist")
    }

    pub fn switch_workspace(&mut self, target: WorkspaceId) -> Option<DesktopEnvironmentEvent> {
        if !self.workspaces.contains_key(&target) || self.active_workspace == target {
            return None;
        }

        let from = self.active_workspace;
        self.active_workspace = target;

        Some(DesktopEnvironmentEvent::WorkspaceSwitched { from, to: target })
    }

    pub fn next_workspace(&mut self) -> Option<DesktopEnvironmentEvent> {
        let keys: Vec<WorkspaceId> = self.workspaces.keys().copied().collect();
        let current_pos = keys.iter().position(|&id| id == self.active_workspace)?;
        let next_pos = (current_pos + 1) % keys.len();
        self.switch_workspace(keys[next_pos])
    }

    pub fn previous_workspace(&mut self) -> Option<DesktopEnvironmentEvent> {
        let keys: Vec<WorkspaceId> = self.workspaces.keys().copied().collect();
        let current_pos = keys.iter().position(|&id| id == self.active_workspace)?;
        let prev_pos = if current_pos == 0 {
            keys.len() - 1
        } else {
            current_pos - 1
        };
        self.switch_workspace(keys[prev_pos])
    }

    pub fn add_window_to_current(&mut self, window_id: WindowId) {
        self.current_workspace_mut().add_window(window_id);
    }

    pub fn remove_window(&mut self, window_id: WindowId) {
        for ws in self.workspaces.values_mut() {
            ws.remove_window(window_id);
        }
    }

    pub fn move_window_to_workspace(&mut self, window_id: WindowId, target: WorkspaceId) -> bool {
        if !self.workspaces.contains_key(&target) {
            return false;
        }

        self.remove_window(window_id);
        if let Some(target_ws) = self.workspaces.get_mut(&target) {
            target_ws.add_window(window_id);
            true
        } else {
            false
        }
    }

    pub fn workspaces(&self) -> impl Iterator<Item = &Workspace> {
        self.workspaces.values()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_manager_switching() {
        let mut wm = WorkspaceManager::new(4);
        assert_eq!(wm.active_workspace_id(), WorkspaceId::new(1));

        let ev = wm.switch_workspace(WorkspaceId::new(2)).unwrap();
        assert_eq!(
            ev,
            DesktopEnvironmentEvent::WorkspaceSwitched {
                from: WorkspaceId::new(1),
                to: WorkspaceId::new(2)
            }
        );
        assert_eq!(wm.active_workspace_id(), WorkspaceId::new(2));

        wm.next_workspace();
        assert_eq!(wm.active_workspace_id(), WorkspaceId::new(3));

        wm.previous_workspace();
        assert_eq!(wm.active_workspace_id(), WorkspaceId::new(2));
    }

    #[test]
    fn test_move_window_between_workspaces() {
        let mut wm = WorkspaceManager::new(4);
        let win = WindowId::new(10);

        wm.add_window_to_current(win);
        assert!(wm.current_workspace().contains_window(win));

        wm.move_window_to_workspace(win, WorkspaceId::new(3));
        assert!(!wm.current_workspace().contains_window(win));

        wm.switch_workspace(WorkspaceId::new(3));
        assert!(wm.current_workspace().contains_window(win));
    }
}
