//! In-memory message bus shared by BabyDraDE features and the compositor.
//!
//! A single `Router` receives decoded envelopes from abstract Unix sockets
//! and writes replies into per-peer outboxes. The compositor uses `BusSender`
//! and `Connection` inside its event loop; wire types and codecs come from
//! `common::protocol`.

pub mod connection;
pub mod router;
pub mod sender;
pub mod socket;

#[cfg(test)]
mod tests;

pub use common::protocol::{
    abstract_addr, codec, connect, decode, encode, envelope, models, verify_peer, writer,
    ActiveWindowInfo, BusError, BusErrorCode, CodecError, Control, Envelope, Event, Incoming, Kind,
    LineBuffer, Message, MessageId, PeerId, PeerName, ProtocolError, Request, Response, Target,
    Topic, WindowAction, WindowTarget, Writer, CONFIG_RELOAD_TOPIC, PEERS_TOPIC, PROTOCOL_VERSION,
    REQUEST_TIMEOUT,
};
pub use connection::{Connection, ReadResult};
pub use router::Router;
pub use sender::BusSender;
