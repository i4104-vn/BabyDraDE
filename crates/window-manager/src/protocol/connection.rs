//! Nonblocking framing helper shared by both sides of the bus socket.

use std::io::{self, ErrorKind, Read, Write};
use std::os::unix::net::UnixStream;

use common::protocol::{decode, encode, CodecError, Envelope, LineBuffer};

/// Result of draining the readable side of a connection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadResult {
    /// Complete envelopes decoded from the read buffer.
    Messages(Vec<Envelope>),
    /// No complete frame is available right now.
    WouldBlock,
    /// The peer closed its end of the socket.
    Closed,
}

/// One nonblocking socket with a line decoder and a buffered writer.
pub struct Connection {
    stream: UnixStream,
    buffer: LineBuffer,
    outbox: Vec<u8>,
}

impl Connection {
    /// Wraps a stream in a nonblocking connection.
    ///
    /// # Arguments
    /// * `stream` - Socket to read and write frames on.
    ///
    /// # Returns
    /// A connection with empty read and write buffers.
    pub fn new(stream: UnixStream) -> Self {
        let _ = stream.set_nonblocking(true);
        Self {
            stream,
            buffer: LineBuffer::new(),
            outbox: Vec::new(),
        }
    }

    /// Returns the underlying socket, for peer-credential checks or shutdown.
    ///
    /// # Returns
    /// A reference to the nonblocking socket.
    pub fn stream(&self) -> &UnixStream {
        &self.stream
    }

    /// Reads all currently available bytes and decodes complete envelopes.
    ///
    /// # Returns
    /// The decoded frames, `WouldBlock` while a line is incomplete, or `Closed`
    /// once the peer hangs up.
    ///
    /// # Errors
    /// Returns an [`ErrorKind::InvalidData`] error for a malformed or oversized
    /// frame; other socket errors are propagated unchanged.
    pub fn read_available(&mut self) -> io::Result<ReadResult> {
        let mut chunk = [0u8; 4096];
        let mut closed = false;
        loop {
            match self.stream.read(&mut chunk) {
                Ok(0) => {
                    closed = true;
                    break;
                }
                Ok(count) => {
                    self.buffer
                        .push(&chunk[..count])
                        .map_err(|error| io::Error::new(ErrorKind::InvalidData, error))?;
                }
                Err(error) if error.kind() == ErrorKind::Interrupted => continue,
                Err(error) if error.kind() == ErrorKind::WouldBlock => break,
                Err(error) => return Err(error),
            }
        }
        let mut messages = Vec::new();
        while let Some(line) = self.buffer.take_line() {
            let envelope =
                decode(&line).map_err(|error| io::Error::new(ErrorKind::InvalidData, error))?;
            messages.push(envelope);
        }
        if !messages.is_empty() {
            return Ok(ReadResult::Messages(messages));
        }
        if closed {
            Ok(ReadResult::Closed)
        } else {
            Ok(ReadResult::WouldBlock)
        }
    }

    /// Appends an encoded envelope to the pending write buffer.
    ///
    /// # Arguments
    /// * `envelope` - Frame to send on the next [`Self::flush`].
    ///
    /// # Returns
    /// `Ok(())` once the encoded frame is buffered.
    ///
    /// # Errors
    /// Returns a codec error when the envelope cannot be serialized.
    pub fn queue(&mut self, envelope: &Envelope) -> Result<(), CodecError> {
        let line = encode(envelope)?;
        self.outbox.extend_from_slice(&line);
        Ok(())
    }

    /// Writes as much of the outbox as the socket accepts.
    ///
    /// # Returns
    /// `true` when the outbox is empty, `false` when the socket is full.
    ///
    /// # Errors
    /// Returns the socket error, except [`ErrorKind::WouldBlock`] which is
    /// reported as `false`.
    pub fn flush(&mut self) -> io::Result<bool> {
        while !self.outbox.is_empty() {
            match self.stream.write(&self.outbox) {
                Ok(0) => {
                    return Err(io::Error::new(
                        ErrorKind::WriteZero,
                        "bus connection closed while flushing",
                    ))
                }
                Ok(count) => {
                    let _ = self.outbox.drain(..count);
                }
                Err(error) if error.kind() == ErrorKind::Interrupted => continue,
                Err(error) if error.kind() == ErrorKind::WouldBlock => return Ok(false),
                Err(error) => return Err(error),
            }
        }
        Ok(true)
    }

    /// Returns `true` while frames are waiting to be written.
    ///
    /// # Returns
    /// `true` when the pending write buffer is non-empty.
    pub fn has_outbox(&self) -> bool {
        !self.outbox.is_empty()
    }
}
