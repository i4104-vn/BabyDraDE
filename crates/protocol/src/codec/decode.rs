//! Envelope deserialization.

use serde::de::DeserializeOwned;

use super::{encode::MAX_MESSAGE_BYTES, error::CodecError};
use crate::models::{Envelope, Payload, PROTOCOL_VERSION};

/// Decodes one newline-terminated JSON line into an envelope.
///
/// # Arguments
/// * `line` - Raw bytes of exactly one message.
///
/// # Returns
/// The decoded envelope, or a codec error. Version mismatches are rejected
/// before the payload is interpreted.
pub fn decode(line: &[u8]) -> Result<Envelope, CodecError> {
    if line.len() > MAX_MESSAGE_BYTES {
        return Err(CodecError::TooLarge {
            size: line.len(),
            limit: MAX_MESSAGE_BYTES,
        });
    }
    let envelope: Envelope = serde_json::from_slice(line)
        .map_err(|error| CodecError::InvalidJson(error.to_string()))?;
    if envelope.version != PROTOCOL_VERSION {
        return Err(CodecError::UnsupportedVersion {
            found: envelope.version,
            expected: PROTOCOL_VERSION,
        });
    }
    Ok(envelope)
}

/// Decodes an arbitrary serializable value (used for response payloads in tests
/// and by transports that carry raw values).
pub fn decode_value<T: DeserializeOwned>(line: &[u8]) -> Result<T, CodecError> {
    serde_json::from_slice(line).map_err(|error| CodecError::InvalidJson(error.to_string()))
}

/// Returns `true` when the envelope carries a request payload.
pub fn is_request(envelope: &Envelope) -> bool {
    matches!(envelope.payload, Payload::Request(_))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Envelope, Payload, Request};

    #[test]
    fn round_trips_an_envelope() {
        let envelope = Envelope::new(42, Payload::Request(Request::Ping));
        let line = crate::codec::encode(&envelope).expect("encoded");
        assert_eq!(decode(&line).expect("decoded"), envelope);
    }

    #[test]
    fn rejects_mismatched_version() {
        let mut envelope = Envelope::new(1, Payload::Request(Request::Ping));
        envelope.version = PROTOCOL_VERSION + 1;
        let line = crate::codec::encode(&envelope).expect("encoded");
        assert!(matches!(
            decode(&line),
            Err(CodecError::UnsupportedVersion { .. })
        ));
    }
}
