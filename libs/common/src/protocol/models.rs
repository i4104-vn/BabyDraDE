//! Serializable request, response and event bodies shared by both sides.

use serde::{Deserialize, Serialize};

use super::MessageId;
use crate::{ShortcutBinding, ShortcutTarget};

/// Metadata describing the focused window.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveWindowInfo {
    /// Wayland app id of the focused toplevel.
    pub app_id: String,
    /// Human-readable window title.
    pub title: String,
    /// Process id, when the compositor could resolve it.
    pub process_id: Option<u32>,
}

impl ActiveWindowInfo {
    /// Returns `true` when neither app id nor title is present.
    pub fn is_empty(&self) -> bool {
        self.app_id.trim().is_empty() && self.title.trim().is_empty()
    }
}

/// Selector for the window a request should act on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WindowTarget {
    /// Currently focused window.
    Focused,
    /// Window whose Wayland app id matches exactly.
    AppId(String),
    /// Window whose title matches exactly.
    Title(String),
}

/// Lifecycle operation a feature asks the compositor to perform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WindowAction {
    /// Close the focused window.
    Close,
    /// Minimize the focused window.
    Minimize,
    /// Toggle maximize on the focused window.
    Maximize,
    /// Toggle fullscreen on the focused window.
    Fullscreen,
}

impl WindowAction {
    /// Parses the legacy plain-text action name into a typed action.
    ///
    /// # Arguments
    /// * `raw` - Action name such as `close` or `minimize`.
    ///
    /// # Returns
    /// The matching action, or `None` for unknown input.
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "close" => Some(Self::Close),
            "minimize" => Some(Self::Minimize),
            "maximize" => Some(Self::Maximize),
            "fullscreen" => Some(Self::Fullscreen),
            _ => None,
        }
    }

    /// Returns the legacy plain-text name of this action.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Close => "close",
            Self::Minimize => "minimize",
            Self::Maximize => "maximize",
            Self::Fullscreen => "fullscreen",
        }
    }
}

/// One feature-to-compositor operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Request {
    /// Perform a window lifecycle action on the focused window.
    WindowAction {
        /// Typed action to perform.
        action: WindowAction,
    },
    /// Close a specific window selected by app id or title.
    CloseTargeted {
        /// Selector for the window to close.
        target: WindowTarget,
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

/// Failure reason reported by the compositor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolError {
    /// Envelope used an unsupported protocol version.
    UnsupportedVersion,
    /// The compositor has no handler for this request kind.
    UnsupportedRequest,
    /// The requested resource does not exist.
    NotFound,
    /// The compositor could not complete the operation.
    Failed(String),
}

/// Result of one request, correlated by the request's [`MessageId`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Response {
    /// The request was accepted.
    Ok {
        /// Id of the request being answered.
        request_id: MessageId,
    },
    /// Shortcut table snapshot.
    Shortcuts {
        /// Id of the request being answered.
        request_id: MessageId,
        /// Every binding with its owning feature.
        entries: Vec<(String, ShortcutBinding)>,
    },
    /// Active window snapshot.
    ActiveWindow {
        /// Id of the request being answered.
        request_id: MessageId,
        /// Focused window metadata, when any.
        info: Option<ActiveWindowInfo>,
    },
    /// The request failed.
    Error {
        /// Id of the request being answered.
        request_id: MessageId,
        /// Failure reason.
        error: ProtocolError,
    },
}

impl Response {
    /// Returns the id of the request this response answers.
    pub fn request_id(&self) -> MessageId {
        match self {
            Self::Ok { request_id }
            | Self::Shortcuts { request_id, .. }
            | Self::ActiveWindow { request_id, .. }
            | Self::Error { request_id, .. } => *request_id,
        }
    }
}

/// One compositor event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Event {
    /// Focus moved to another window.
    ActiveWindowChanged {
        /// New focused window metadata.
        info: ActiveWindowInfo,
    },
    /// A feature published a shortcut table revision.
    ShortcutsChanged {
        /// Owner whose bindings changed.
        owner: String,
        /// New revision number.
        revision: u64,
    },
    /// The compositor event loop is running.
    CompositorReady,
    /// The compositor is shutting down.
    Shutdown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_legacy_action_names() {
        assert_eq!(WindowAction::parse("close"), Some(WindowAction::Close));
        assert_eq!(
            WindowAction::parse(" MINIMIZE "),
            Some(WindowAction::Minimize)
        );
        assert_eq!(
            WindowAction::parse("maximize"),
            Some(WindowAction::Maximize)
        );
        assert_eq!(
            WindowAction::parse("fullscreen"),
            Some(WindowAction::Fullscreen)
        );
        assert_eq!(WindowAction::parse("unknown"), None);
    }

    #[test]
    fn round_trips_action_name() {
        for action in [
            WindowAction::Close,
            WindowAction::Minimize,
            WindowAction::Maximize,
            WindowAction::Fullscreen,
        ] {
            assert_eq!(WindowAction::parse(action.as_str()), Some(action));
        }
    }
}
