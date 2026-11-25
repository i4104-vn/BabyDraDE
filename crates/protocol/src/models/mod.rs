//! Wire data model. Only plain, serializable data lives here.

pub mod envelope;
pub mod event;
pub mod request;
pub mod response;
pub mod types;

pub use envelope::{Envelope, MessageId, Payload, PROTOCOL_VERSION};
pub use event::Event;
pub use request::Request;
pub use response::{ProtocolError, Response};
pub use types::{ActiveWindowInfo, WindowAction, WindowTarget};
