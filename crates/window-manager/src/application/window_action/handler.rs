//! Typed window actions crossing the compositor boundary.
//!
//! External producers (dock, topbar, IPC helpers) still send plain strings,
//! but the string is parsed once at the boundary. The rest of the crate deals
//! with an exhaustive enum, so unknown actions cannot be silently ignored
//! by an exhaustive match deep inside the adapter layer.

use tracing::warn;

/// A window-management command requested from outside the compositor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WindowAction {
    Close,
    Minimize,
    Maximize,
    Fullscreen,
}

impl WindowAction {
    /// Parses a raw action name coming from an external producer.
    ///
    /// Unknown values are logged and discarded here, at the boundary.
    pub(crate) fn parse(raw: &str) -> Option<Self> {
        match raw.trim() {
            "close" => Some(Self::Close),
            "minimize" => Some(Self::Minimize),
            "maximize" => Some(Self::Maximize),
            "fullscreen" => Some(Self::Fullscreen),
            other => {
                warn!(action = other, "ignoring unknown window action");
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_all_known_actions() {
        assert_eq!(WindowAction::parse("close"), Some(WindowAction::Close));
        assert_eq!(
            WindowAction::parse("minimize"),
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
    }

    #[test]
    fn rejects_unknown_actions() {
        assert_eq!(WindowAction::parse("resize"), None);
        assert_eq!(WindowAction::parse(""), None);
    }
}
