//! Shared enums, constants, and common types for BabyDraDE applications and compositor.

/// Serde default for `bool` fields that are enabled unless explicitly disabled.
///
/// Keeping one definition avoids the many per-module copies of this helper.
#[inline]
pub fn default_true() -> bool {
    true
}

pub mod constants;
pub mod enums;
/// Serializable file chooser contracts shared across crates.
pub mod filedialog;
/// Shared wire protocol types, codec and socket helpers (requires the `serde` feature).
#[cfg(feature = "serde")]
pub mod protocol;
/// Shared shortcut registration protocol (requires the `serde` feature).
#[cfg(feature = "serde")]
pub mod shortcuts;

pub use constants::*;
pub use enums::*;
pub use filedialog::*;
#[cfg(feature = "serde")]
pub use shortcuts::*;
