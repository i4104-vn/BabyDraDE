/// Authentication helpers for lock screen and session checks.
pub mod auth;
/// Configuration paths, loaders, savers and models.
pub mod config;
/// Desktop entries, file index, screenshots and wallpaper.
pub mod desktop;
/// Feature-side protocol bus client and shared wire types.
pub mod protocol;
/// Shared device sensors, including orientation drivers.
pub mod sensors;
/// System monitoring and control services.
pub mod system;
/// UI support: translations, theme/cursor discovery and tray host.
pub mod ui;

// Stable re-exports keep existing `services::*` paths working after regrouping.
pub use config::models;
pub use desktop::{files, screenshot, wallpaper};
pub use system::filedialog;
pub use ui::{i18n, theme, tray};

/// Translates a key into the active locale.
pub use i18n::trans;
