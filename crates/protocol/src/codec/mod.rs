//! Encoding and decoding of protocol envelopes.
//!
//! One envelope is exactly one JSON line terminated by `\n`. This keeps the
//! framing trivial to debug (`cat` the queue directory), avoids partial reads,
//! and lets future transports reuse the same codec.

pub mod decode;
pub mod encode;
pub mod error;

pub use decode::decode;
pub use encode::encode;
pub use error::CodecError;
