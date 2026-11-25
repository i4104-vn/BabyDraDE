//! Compositor-initiated notifications delivered to every feature.

use serde::{Deserialize, Serialize};

use super::ActiveWindowInfo;

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
