pub mod system_cmds;
pub mod window_cmds;
pub mod workspace_cmds;

pub use system_cmds::SystemCommand;
pub use window_cmds::WindowCommand;
pub use workspace_cmds::WorkspaceCommand;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Command {
    Window(WindowCommand),
    Workspace(WorkspaceCommand),
    System(SystemCommand),
}

impl From<WindowCommand> for Command {
    fn from(cmd: WindowCommand) -> Self {
        Self::Window(cmd)
    }
}

impl From<WorkspaceCommand> for Command {
    fn from(cmd: WorkspaceCommand) -> Self {
        Self::Workspace(cmd)
    }
}

impl From<SystemCommand> for Command {
    fn from(cmd: SystemCommand) -> Self {
        Self::System(cmd)
    }
}
