use crate::entities::Output;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompositorError {
    BackendInitializationFailed(String),
    OutputConfigurationFailed(String),
    BackendTerminated,
}

impl std::fmt::Display for CompositorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BackendInitializationFailed(msg) => {
                write!(f, "Compositor backend init failed: {msg}")
            }
            Self::OutputConfigurationFailed(msg) => {
                write!(f, "Output configuration failed: {msg}")
            }
            Self::BackendTerminated => write!(f, "Compositor backend was terminated"),
        }
    }
}

impl std::error::Error for CompositorError {}

/// Contract for compositor display backends (e.g. Winit for development, DRM/KMS for production).
pub trait CompositorBackend {
    /// Friendly backend name (e.g. "winit" or "drm").
    fn name(&self) -> &str;

    /// Whether this is a direct DRM hardware backend.
    fn is_drm(&self) -> bool;

    /// Initializes backend hardware or window connection.
    fn initialize(&mut self) -> Result<(), CompositorError>;

    /// Returns list of currently active display outputs.
    fn outputs(&self) -> Vec<Output>;
}
