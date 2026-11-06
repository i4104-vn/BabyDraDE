use domain::{WindowId, WorkspaceId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkspaceCommand {
    SwitchWorkspace {
        workspace_id: WorkspaceId,
    },
    MoveWindowToWorkspace {
        window_id: WindowId,
        workspace_id: WorkspaceId,
    },
    NextWorkspace,
    PreviousWorkspace,
}
