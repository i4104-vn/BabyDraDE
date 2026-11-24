/// Authentication helpers for lock screen and session checks.
pub mod auth;
/// Configuration paths, loaders and savers.
pub mod config;
/// Desktop entry discovery.
pub mod desktop;
/// File indexing service.
pub mod files;
/// Internationalization and translation.
pub mod i18n;
/// Serializable configuration models.
pub mod models;
/// Screenshot capture and persistence helpers.
pub mod screenshot;
/// Backward-compatible alias for [`config`].
pub mod settings {
    pub use crate::config::*;
}
/// System monitoring and control services.
pub mod system;
/// GTK theme, icon and cursor discovery.
pub mod theme;
/// Status Notifier tray host.
pub mod tray;
/// Wallpaper mode and path helpers.
pub mod wallpaper;

/// Translates a key into the active locale.
pub use i18n::trans;
