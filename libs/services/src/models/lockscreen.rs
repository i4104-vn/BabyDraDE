//! Lockscreen appearance and user identity configuration models.

use serde::{Deserialize, Serialize};

/// Lockscreen configuration persisted in `~/.babydra/babydra.conf` under `[lockscreen]`.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct LockscreenConfig {
    /// Path to background wallpaper or solid color for the lock screen.
    #[serde(default)]
    pub background: String,
    /// Path to user avatar image displayed on the authentication prompt.
    #[serde(default)]
    pub avatar: String,
}
