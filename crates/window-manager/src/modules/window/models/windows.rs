//! Typed window actions crossing the compositor boundary.
//!
//! Feature apps send the same [`WindowAction`] values over the in-memory bus;
//! the compositor applies them directly instead of translating between two
//! identical enums.

pub(crate) use common::protocol::WindowAction;
