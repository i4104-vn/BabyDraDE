//! Feature-side bus client declarations and re-exports.
//!
//! Implementation lives in focused siblings: `state` owns the reader state
//! and frame routing, `bus` owns connection lifecycle, and `bus_io` owns
//! the public request/publish API.
//!
//! The wire types and constants are re-exported from the external `common`
//! crate so feature crates can name them through `services::protocol` without
//! adding a direct `common` dependency. Narrowing these aliases would break
//! those callers, so they are kept as part of the stable surface.

mod bus;
mod bus_io;
mod state;

#[cfg(test)]
mod tests;

pub use bus::Bus;
pub use common::protocol::{
    ActiveWindowInfo, BusError, BusErrorCode, Control, Envelope, Event, Incoming, Kind, Message,
    MessageId, PeerName, ProtocolError, Request, Response, Target, Topic, WindowAction,
    WindowTarget, CONFIG_RELOAD_TOPIC, PROTOCOL_VERSION,
};
