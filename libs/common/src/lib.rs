//! Shared enums, constants, and common types for BabyDraDE applications and compositor.

pub mod constants;
pub mod enums;
/// Shared wire protocol types, codec and socket helpers (requires the `serde` feature).
#[cfg(feature = "serde")]
pub mod protocol;
/// Shared shortcut registration protocol (requires the `serde` feature).
#[cfg(feature = "serde")]
pub mod shortcuts;

pub use constants::*;
pub use enums::*;
#[cfg(feature = "serde")]
pub use shortcuts::*;
