//! Explicit implementations of the shared ports for `BabydraState`.
//!
//! The implementations used to be assembled from five macro fragments; each
//! port is now implemented in one place so the control flow is readable.

pub(crate) mod overlay;
pub(crate) mod seat;
pub(crate) mod system;
pub(crate) mod window;
