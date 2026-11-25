//! Envelope serialization.

use serde::Serialize;

use super::error::CodecError;

/// Maximum encoded size of one message (1 MiB).
pub const MAX_MESSAGE_BYTES: usize = 1024 * 1024;

/// Encodes an envelope as one newline-terminated JSON line.
///
/// # Arguments
/// * `envelope` - Message to serialize.
///
/// # Returns
/// UTF-8 bytes ending with `\n`, or a codec error.
pub fn encode<T: Serialize>(envelope: &T) -> Result<Vec<u8>, CodecError> {
    let mut bytes = serde_json::to_vec(envelope)
        .map_err(|error| CodecError::InvalidJson(error.to_string()))?;
    if bytes.len() > MAX_MESSAGE_BYTES {
        return Err(CodecError::TooLarge {
            size: bytes.len(),
            limit: MAX_MESSAGE_BYTES,
        });
    }
    bytes.push(b'\n');
    Ok(bytes)
}
