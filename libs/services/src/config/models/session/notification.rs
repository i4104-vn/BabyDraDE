//! Notification daemon and banner configuration models.

use serde::{Deserialize, Serialize};

/// Notification daemon configuration persisted in `~/.babydra/babydra.conf` under `[notification]`.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct NotificationConfig {
    /// Whether "Do Not Disturb" (DND) mode is active, suppressing desktop alerts.
    #[serde(default)]
    pub dnd: bool,
}
