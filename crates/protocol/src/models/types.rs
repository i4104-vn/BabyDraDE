//! Small value types shared by both sides of the bridge.

use serde::{Deserialize, Serialize};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_legacy_action_names() {
        assert_eq!(WindowAction::parse("close"), Some(WindowAction::Close));
        assert_eq!(WindowAction::parse(" MINIMIZE "), Some(WindowAction::Minimize));
        assert_eq!(WindowAction::parse("maximize"), Some(WindowAction::Maximize));
        assert_eq!(WindowAction::parse("fullscreen"), Some(WindowAction::Fullscreen));
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
