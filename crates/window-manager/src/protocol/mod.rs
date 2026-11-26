//! In-memory message bus shared by BabyDraDE features and the compositor.
//!
//! A single [`Router`] receives decoded envelopes from abstract Unix sockets
//! and writes replies into per-peer outboxes. The compositor uses [`BusSender`]
//! and [`Connection`] inside its event loop; wire types and codecs come from
//! `common::protocol`.

pub mod router;
pub mod connection;
pub mod socket;
pub mod sender;

#[cfg(test)]
mod tests;

pub use common::protocol::*;
pub use connection::{Connection, ReadResult};
pub use router::Router;
pub use sender::BusSender;
