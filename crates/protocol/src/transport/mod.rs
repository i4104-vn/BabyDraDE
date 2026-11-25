//! Message transport abstraction and its filesystem implementation.

pub mod file;
pub mod paths;

pub use file::FileTransport;
pub use paths::{protocol_dir, runtime_dir};

/// A bidirectional, best-effort transport for protocol envelopes.
///
/// Implementations must tolerate concurrent writers and must never return a
/// partially written message. `FileTransport` is the default because it needs
/// no extra process privileges and shares the runtime directory used by the
/// rest of BabyDraDE; a Unix-socket implementation can replace it later
/// without touching codecs or payloads.
pub trait Transport {
    /// Writes one encoded envelope to the peer.
    fn send(&self, bytes: &[u8]) -> Result<(), std::io::Error>;

    /// Drains all pending envelopes, oldest first.
    fn drain(&self) -> Result<Vec<Vec<u8>>, std::io::Error>;
}
