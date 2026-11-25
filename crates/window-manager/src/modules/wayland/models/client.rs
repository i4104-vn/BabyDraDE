//! Wayland client state model.
//!
//! `ClientState` is owned by the application state facade so that it can be
//! installed when a client connects; it is re-exported here to keep the
//! Wayland model surface in one place.

pub(crate) use crate::app::models::ClientState;
