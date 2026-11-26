//! Shared wire protocol for the BabyDraDE in-memory message bus.
//!
//! Every frame is one newline-terminated JSON [`Envelope`]. The router stamps
//! `source`, correlates requests with responses and owns delivery policy; peers
//! only fill in `target`, `kind` and `payload`. The types, codec and socket
//! helpers here are shared by the compositor and every feature process.

pub mod codec;
pub mod envelope;
pub mod models;
pub mod socket;
pub mod writer;

pub use codec::{decode, encode, CodecError, LineBuffer};
pub use envelope::{
    BusError, BusErrorCode, Control, Envelope, Incoming, Kind, Message, MessageId, PeerId,
    PeerName, Target, Topic, PROTOCOL_VERSION,
};
pub use models::{
    ActiveWindowInfo, Event, ProtocolError, Request, Response, WindowAction, WindowTarget,
};
pub use socket::{abstract_addr, connect, verify_peer};
pub use writer::Writer;
