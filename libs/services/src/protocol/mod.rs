//! Feature-side bus client declarations and re-exports.
//!
//! Implementation lives in focused siblings: [`state`] owns the reader state
//! and frame routing, [`bus`] owns connection lifecycle, and [`bus_io`] owns
//! the public request/publish API.

mod bus;
mod bus_io;
mod state;

#[cfg(test)]
mod tests;

pub use bus::Bus;
pub use common::protocol::{
    ActiveWindowInfo, BusError, BusErrorCode, Control, Envelope, Event, Incoming, Kind, Message,
    MessageId, PeerName, ProtocolError, Request, Response, Target, Topic, WindowAction,
    WindowTarget, PROTOCOL_VERSION,
};
