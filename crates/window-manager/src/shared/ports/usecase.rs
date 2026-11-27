//! Use-case ports implemented by the composition root's adapters.

use std::ffi::OsStr;

use crate::core::shortcuts::Shortcut;

use super::{ConfigError, GesturePolicy, LaunchError};

/// Port that loads the shortcut table (implemented by the config adapter).
///
/// Unlike a bare function pointer, this trait returns a structured error so
/// the application layer can decide how to degrade and tests can fake it.
pub(crate) trait ShortcutRepository {
    fn load(&self) -> Result<Vec<Shortcut>, ConfigError>;
}

/// Port that launches a configured command (implemented by infrastructure).
///
/// Failures are returned to the caller instead of being swallowed, which
/// makes the outcome observable and easy to fake in tests.
pub(crate) trait CommandLauncher {
    fn launch(&self, command: &str) -> Result<(), LaunchError>;
}

/// Port that starts one autostart command on the compositor's Wayland display.
///
/// The handle returned on success is kept by the caller so it can be
/// terminated when the compositor exits. The application layer does not need
/// to know what kind of handle the infrastructure uses.
pub(crate) trait AutostartLauncher {
    /// Handle to a successfully started process.
    type Child;

    /// Run `command` through the infrastructure with `WAYLAND_DISPLAY` set to
    /// `wayland_display`.
    fn spawn(&self, command: &str, wayland_display: &OsStr) -> Result<Self::Child, LaunchError>;
}

/// Port that loads a fresh gesture policy snapshot.
pub(crate) type LoadGesturePolicy = fn() -> GesturePolicy;

/// Minimal view of the compositor's outputs needed by the rotation feature.
///
/// The rotation controller depends on this port instead of the concrete state
/// facade, so its debounce and baseline rules can be tested without a
/// compositor. Implementations apply one transform per output and a separate
/// transform for absolute touch input.
pub(crate) trait RotationTarget {
    /// Names of the outputs currently attached to the compositor.
    fn output_names(&self) -> Vec<String>;

    /// Fixed transform configured for an output, if any.
    fn configured_transform(&self, name: &str) -> Option<smithay::utils::Transform>;

    /// Transform an output currently advertises, if it is still attached.
    fn current_transform(&self, name: &str) -> Option<smithay::utils::Transform>;

    /// Applies the computed transforms to the named outputs and to touch input.
    fn apply_transforms(
        &mut self,
        transforms: &[(String, smithay::utils::Transform)],
        touch: smithay::utils::Transform,
    );
}
