//! Pure in-memory message router.
//!
//! The router owns no sockets: a caller feeds it decoded `Envelope`s and
//! drains per-peer outboxes. All delivery, correlation, retention and timeout
//! policy lives here so it can be tested without an event loop.

mod delivery;
mod lifecycle;
mod routing;
mod state;

pub use state::Router;

// Used by the protocol's router tests, which live outside this subtree.
#[cfg(test)]
pub(crate) use state::{OFFLINE_LIMIT, PEERS_TOPIC};
