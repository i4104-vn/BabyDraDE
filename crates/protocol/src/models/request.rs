//! Requests a feature can send to the compositor.

use common::{ShortcutBinding, ShortcutTarget};
use serde::{Deserialize, Serialize};

/// One feature-to-compositor operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Request {
    /// Perform a window lifecycle action on the focused window.
    WindowAction {
        /// Typed action to perform.
        action: super::WindowAction,
    },
    /// Close a specific window selected by app id or title.
    CloseTargeted {
        /// Selector for the window to close.
        target: super::WindowTarget,
    },
    /// Add or replace one shortcut binding owned by the sending feature.
    ShortcutRegister {
        /// Stable feature id, for example `org.babydrade.topbar`.
        owner: String,
        /// Binding to publish.
        binding: ShortcutBinding,
    },
    /// Remove one shortcut binding by id.
    ShortcutUnregister {
        /// Owner of the binding.
        owner: String,
        /// Id of the binding to remove.
        id: String,
    },
    /// Return every registered shortcut binding with its owner.
    ShortcutList,
    /// Run a shell command through the compositor launcher.
    RunCommand {
        /// Command string to execute.
        command: String,
    },
    /// Ask the compositor to trigger a built-in action.
    SystemAction {
        /// Built-in action to trigger.
        target: ShortcutTarget,
    },
    /// Liveness probe used by features during startup.
    Ping,
}
