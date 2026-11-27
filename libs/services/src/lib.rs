//! Shared desktop services for configuration, system monitoring, UI support, sensors, and bus messaging across BabyDraDE crates.
//!
//! The module tree is grouped by use case rather than by kind: [`auth`] handles
//! lock-screen checks, [`config`] owns settings paths and loaders, [`desktop`]
//! covers file indexing and capture, [`protocol`] owns the bus client and wire
//! types, [`sensors`] reads device orientation, [`system`] collects host metrics
//! and native dialogs, and [`ui`] provides translations, theming and the tray
//! host. Public items are re-exported individually so callers can depend on this
//! crate alone.

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

// Compatibility aliases: downstream crates use these shorter paths (for example
// `services::models::MouseConfig` and `services::trans(...)`) instead of the
// grouped module paths above. They are plain re-exports of existing items, so
// keeping them does not duplicate any API; removing them would break callers.
pub use config::models;
pub use desktop::{files, screenshot, wallpaper};
pub use system::filedialog;
pub use ui::{i18n, theme, tray};

/// Looks up `key` in the active locale, returning the key itself when untranslated.
pub use i18n::trans;
