//! Encoding and decoding of bus envelopes.
//!
//! One envelope is exactly one newline-terminated JSON line; the same codec is
//! shared by the router and both socket sides.

use std::fmt;

use serde::Serialize;

use super::{Envelope, PROTOCOL_VERSION};

/// Maximum encoded size of one message (1 MiB).
pub const MAX_MESSAGE_BYTES: usize = 1024 * 1024;

/// Largest frame the buffer accepts: one encoded message plus its newline.
pub(crate) const MAX_FRAME_BYTES: usize = MAX_MESSAGE_BYTES + 1;

/// Encodes an envelope as one newline-terminated JSON line.
///
/// # Arguments
/// * `envelope` - Message to serialize.
///
/// # Returns
/// UTF-8 bytes ending with `\n`, or a codec error.
///
/// # Errors
/// Returns [`CodecError::InvalidJson`] when serialization fails or
/// [`CodecError::TooLarge`] when the encoded message exceeds the limit.
pub fn encode<T: Serialize>(envelope: &T) -> Result<Vec<u8>, CodecError> {
    let mut bytes =
        serde_json::to_vec(envelope).map_err(|error| CodecError::InvalidJson(error.to_string()))?;
    if bytes.len() > MAX_MESSAGE_BYTES {
        return Err(CodecError::TooLarge {
            size: bytes.len(),
            limit: MAX_MESSAGE_BYTES,
        });
    }
    bytes.push(b'\n');
    Ok(bytes)
}

/// Decodes one newline-terminated JSON line into an envelope.
///
/// # Arguments
/// * `line` - Raw bytes of exactly one message, without the newline.
///
/// # Returns
/// The decoded envelope.
///
/// # Errors
/// Returns [`CodecError::TooLarge`] for oversized lines,
/// [`CodecError::InvalidJson`] for malformed lines and
/// [`CodecError::UnsupportedVersion`] for a version mismatch.
pub fn decode(line: &[u8]) -> Result<Envelope, CodecError> {
    if line.len() > MAX_MESSAGE_BYTES {
        return Err(CodecError::TooLarge {
            size: line.len(),
            limit: MAX_MESSAGE_BYTES,
        });
    }
    let envelope: Envelope =
        serde_json::from_slice(line).map_err(|error| CodecError::InvalidJson(error.to_string()))?;
    if !envelope.is_current() {
        return Err(CodecError::UnsupportedVersion {
            found: envelope.version,
            expected: PROTOCOL_VERSION,
        });
    }
    Ok(envelope)
}

/// Protocol codec failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodecError {
    /// JSON serialization or deserialization failed.
    InvalidJson(String),
    /// The transport could not deliver the message.
    Io(String),
    /// Envelope carried a version this process does not understand.
    UnsupportedVersion {
        /// Version found on the wire.
        found: u16,
        /// Version expected by this process.
        expected: u16,
    },
    /// Encoded message exceeded the safety limit.
    TooLarge {
        /// Actual size in bytes.
        size: usize,
        /// Allowed size in bytes.
        limit: usize,
    },
}

impl fmt::Display for CodecError {
    /// Writes a human-readable description of this failure.
    ///
    /// # Arguments
    /// * `f` - Formatter receiving the description.
    ///
    /// # Errors
    /// Returns the underlying formatter error, if any.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidJson(message) => write!(f, "invalid protocol JSON: {message}"),
            Self::Io(message) => write!(f, "protocol transport error: {message}"),
            Self::UnsupportedVersion { found, expected } => {
                write!(f, "protocol version {found} (expected {expected})")
            }
            Self::TooLarge { size, limit } => write!(
                f,
                "protocol message too large: {size} bytes (limit {limit})"
            ),
        }
    }
}

impl std::error::Error for CodecError {}

/// Reassembles newline-terminated JSON lines from arbitrary socket reads.
#[derive(Debug, Default)]
pub struct LineBuffer {
    pending: Vec<u8>,
    /// Set after an oversized unterminated prefix: bytes are skipped until the
    /// next newline so the connection can resynchronise with the next frame.
    discarding: bool,
}

impl LineBuffer {
    /// Creates an empty buffer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends bytes read from a socket.
    ///
    /// # Arguments
    /// * `bytes` - Next chunk of raw bytes, possibly ending mid-line.
    ///
    /// # Errors
    /// Returns [`CodecError::TooLarge`] when the buffer would exceed the frame
    /// limit. Buffered complete lines are preserved; an oversized unterminated
    /// prefix is skipped up to the next newline so later frames can still be read.
    pub fn push(&mut self, bytes: &[u8]) -> Result<(), CodecError> {
        let mut rest = bytes;
        let mut result = Ok(());

        while !rest.is_empty() {
            if self.discarding {
                // Skip the remainder of the oversized frame, including its newline.
                match rest.iter().position(|byte| *byte == b'\n') {
                    Some(end) => {
                        self.discarding = false;
                        rest = &rest[end + 1..];
                    }
                    None => return result,
                }
                continue;
            }

            // Work one newline at a time so a rejected frame cannot poison the
            // complete lines that follow it in the same read.
            let (chunk, tail) = match rest.iter().position(|byte| *byte == b'\n') {
                Some(end) => (&rest[..=end], &rest[end + 1..]),
                None => (rest, &[][..]),
            };
            let size = self.pending.len().saturating_add(chunk.len());
            if size > MAX_FRAME_BYTES {
                result = Err(CodecError::TooLarge {
                    size,
                    limit: MAX_FRAME_BYTES,
                });
                if chunk.last() == Some(&b'\n') {
                    // The oversized line is complete: drop it and keep reading.
                    // An unterminated prefix already in the buffer belongs to
                    // that same frame; complete lines queued before it stay.
                    if !self.pending.contains(&b'\n') {
                        self.pending.clear();
                    }
                    rest = tail;
                    continue;
                }
                // Unterminated oversized prefix: wait for its newline in a
                // later read before accepting any new bytes.
                self.discarding = true;
                return result;
            }
            self.pending.extend_from_slice(chunk);
            rest = tail;
        }
        result
    }

    /// Removes and returns the next complete line, without its terminator.
    ///
    /// # Returns
    /// The next line with `\n` and an optional `\r` stripped, or `None` while
    /// no complete line is buffered.
    pub fn take_line(&mut self) -> Option<Vec<u8>> {
        let end = self.pending.iter().position(|byte| *byte == b'\n')?;
        let mut line: Vec<u8> = self.pending.drain(..=end).collect();
        line.pop();
        if line.last() == Some(&b'\r') {
            line.pop();
        }
        Some(line)
    }

    /// Returns `true` when no bytes are buffered.
    ///
    /// # Returns
    /// `true` when the pending buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{Kind, Target};

    #[test]
    fn round_trips_a_current_envelope() {
        let envelope = Envelope::new(7, Target::Broadcast, Kind::Event, serde_json::json!(42));
        let line = encode(&envelope).expect("encoded");
        assert_eq!(decode(&line).expect("decoded"), envelope);
    }

    #[test]
    fn rejects_malformed_json() {
        assert!(matches!(
            decode(b"{not json"),
            Err(CodecError::InvalidJson(_))
        ));
    }

    #[test]
    fn rejects_an_oversized_line() {
        let line = vec![b'x'; MAX_MESSAGE_BYTES + 1];
        assert!(matches!(decode(&line), Err(CodecError::TooLarge { .. })));
    }

    #[test]
    fn rejects_a_wrong_protocol_version() {
        let mut envelope = Envelope::new(1, Target::Router, Kind::Request, serde_json::json!({}));
        envelope.version = PROTOCOL_VERSION + 1;
        let line = encode(&envelope).expect("encoded");
        assert!(matches!(
            decode(&line),
            Err(CodecError::UnsupportedVersion { .. })
        ));
    }

    #[test]
    fn holds_a_partial_line_until_it_is_complete() {
        let mut buffer = LineBuffer::new();
        assert_eq!(buffer.push(b"{\"id\": 1"), Ok(()));
        assert_eq!(buffer.take_line(), None);
        assert_eq!(buffer.push(b"}\n"), Ok(()));
        assert_eq!(buffer.take_line(), Some(b"{\"id\": 1}".to_vec()));
    }

    #[test]
    fn splits_several_lines_from_one_chunk() {
        let mut buffer = LineBuffer::new();
        assert_eq!(buffer.push(b"first\nsecond\nthird\n"), Ok(()));
        assert_eq!(buffer.take_line(), Some(b"first".to_vec()));
        assert_eq!(buffer.take_line(), Some(b"second".to_vec()));
        assert_eq!(buffer.take_line(), Some(b"third".to_vec()));
        assert_eq!(buffer.take_line(), None);
    }

    #[test]
    fn rejects_an_oversized_chunk() {
        let mut buffer = LineBuffer::new();
        let chunk = vec![b'x'; MAX_FRAME_BYTES + 1];
        assert!(matches!(
            buffer.push(&chunk),
            Err(CodecError::TooLarge { .. })
        ));
    }

    #[test]
    fn keeps_buffered_line_after_an_oversized_chunk() {
        let mut buffer = LineBuffer::new();
        let _ = buffer.push(b"ok\n");
        let chunk = vec![b'x'; MAX_FRAME_BYTES];
        let _ = buffer.push(&chunk);
        assert_eq!(buffer.take_line(), Some(b"ok".to_vec()));
    }

    #[test]
    fn resynchronises_after_an_oversized_unterminated_frame() {
        let mut buffer = LineBuffer::new();
        let oversized = vec![b'x'; MAX_FRAME_BYTES + 1];
        assert!(matches!(
            buffer.push(&oversized),
            Err(CodecError::TooLarge { .. })
        ));
        // The terminator of the rejected frame must not poison the next one.
        assert_eq!(buffer.push(b"\n"), Ok(()));
        assert_eq!(buffer.take_line(), None);
        assert_eq!(buffer.push(b"next\n"), Ok(()));
        assert_eq!(buffer.take_line(), Some(b"next".to_vec()));
    }

    #[test]
    fn terminator_after_a_full_buffer_does_not_stall_the_stream() {
        let mut buffer = LineBuffer::new();
        // A frame exactly at the buffer limit leaves no room for its newline.
        let full = vec![b'x'; MAX_FRAME_BYTES];
        assert_eq!(buffer.push(&full), Ok(()));
        assert!(matches!(
            buffer.push(b"\n"),
            Err(CodecError::TooLarge { .. })
        ));
        assert_eq!(buffer.push(b"good\n"), Ok(()));
        assert_eq!(buffer.take_line(), Some(b"good".to_vec()));
    }

    #[test]
    fn resynchronises_when_an_oversized_frame_arrives_with_its_newline() {
        let mut buffer = LineBuffer::new();
        let mut oversized = vec![b'x'; MAX_MESSAGE_BYTES + 1];
        oversized.push(b'\n');
        assert!(matches!(
            buffer.push(&oversized),
            Err(CodecError::TooLarge { .. })
        ));
        assert_eq!(buffer.push(b"good\n"), Ok(()));
        assert_eq!(buffer.take_line(), Some(b"good".to_vec()));
    }
}
