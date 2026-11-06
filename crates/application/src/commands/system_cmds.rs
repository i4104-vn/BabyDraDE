use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SystemCommand {
    LaunchApp { command_or_desktop_id: String },
    ReloadConfig,
    QuitCompositor,
}
