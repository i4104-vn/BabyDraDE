//! Shared socket writer with collision-free ids and timed frame delivery.

use std::io::{self, ErrorKind, Write};
use std::net::Shutdown;
use std::os::unix::net::UnixStream;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::Serialize;

use super::codec::encode;
use super::{Envelope, MessageId};

/// Maximum time a writer waits for a nonblocking socket to accept a frame.
pub const WRITE_TIMEOUT: Duration = Duration::from_secs(5);

/// Session-wide seed counter mixed with pid and time for each writer.
static WRITER_SEED: AtomicU64 = AtomicU64::new(0);

/// Locks a mutex, recovering from a poisoned lock instead of panicking.
///
/// Recovery matters for the shared socket writer: a panic while holding it
/// must not deadlock every later frame. The event is reported so the upstream
/// panic remains visible.
///
/// # Arguments
/// * `mutex` - Mutex to acquire.
///
/// # Returns
/// A guard for the locked value.
pub fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poisoned| {
        eprintln!("recovering a poisoned bus writer mutex after a panic");
        poisoned.into_inner()
    })
}

/// Serializes a message body into the JSON payload of an envelope.
///
/// # Arguments
/// * `body` - Value convertible to JSON.
///
/// # Returns
/// The serialized JSON payload.
///
/// # Errors
/// Returns an `InvalidInput` error when the body cannot be serialized.
pub fn to_value<T: Serialize>(body: &T) -> io::Result<serde_json::Value> {
    serde_json::to_value(body).map_err(|error| io::Error::new(ErrorKind::InvalidInput, error))
}

/// Builds an id space that cannot collide between concurrent peer processes.
///
/// # Returns
/// A seed mixed from the process id, the current time and a session counter.
fn seed_id() -> u64 {
    let pid = std::process::id() as u64;
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos() as u64)
        .unwrap_or(0);
    let sequence = WRITER_SEED.fetch_add(1, Ordering::Relaxed);
    pid.wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(nanos)
        .wrapping_add(sequence.wrapping_mul(0xBF58_476D_1CE4_E5B9))
}

/// Writes one encoded envelope to a stream under the writer lock.
///
/// Sockets shared with a nonblocking reader can refuse part of a frame, so
/// writes advance by the number of bytes actually accepted and retry within
/// [`WRITE_TIMEOUT`].
///
/// # Arguments
/// * `stream` - Socket guarded by the writer mutex.
/// * `envelope` - Frame to encode and send.
///
/// # Errors
/// Returns an invalid-data error when encoding fails, a `WriteZero` error
/// when the peer closes mid-frame, or the socket error after the deadline.
pub(crate) fn write_frame(stream: &Mutex<UnixStream>, envelope: &Envelope) -> io::Result<()> {
    let line = encode(envelope).map_err(|error| io::Error::new(ErrorKind::InvalidData, error))?;
    let mut stream = lock(stream);
    let started = Instant::now();
    let mut sent = 0;
    while sent < line.len() {
        match stream.write(&line[sent..]) {
            Ok(0) => {
                return Err(io::Error::new(
                    ErrorKind::WriteZero,
                    "bus connection closed while writing",
                ))
            }
            Ok(count) => sent += count,
            Err(error) if error.kind() == ErrorKind::Interrupted => continue,
            Err(error) if error.kind() == ErrorKind::WouldBlock => {
                if started.elapsed() >= WRITE_TIMEOUT {
                    return Err(error);
                }
                thread::sleep(Duration::from_millis(1));
            }
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

/// A cloneable bus socket writer with a private, process-unique id sequence.
#[derive(Clone)]
pub struct Writer {
    stream: Arc<Mutex<UnixStream>>,
    next_id: Arc<AtomicU64>,
}

impl Writer {
    /// Wraps a connected stream and prepares it for timed writes.
    ///
    /// # Arguments
    /// * `stream` - Connected bus socket.
    ///
    /// # Returns
    /// A writer with a process-unique id sequence.
    pub fn new(stream: UnixStream) -> Self {
        let _ = stream.set_write_timeout(Some(WRITE_TIMEOUT));
        Self {
            stream: Arc::new(Mutex::new(stream)),
            next_id: Arc::new(AtomicU64::new(seed_id())),
        }
    }

    /// Allocates the next id in this writer's private space.
    ///
    /// # Returns
    /// A monotonically increasing correlation id.
    pub fn allocate_id(&self) -> MessageId {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    /// Encodes and writes one envelope to the shared socket.
    ///
    /// # Arguments
    /// * `envelope` - Frame to serialize and send.
    ///
    /// # Errors
    /// Returns an encoding or socket error when the frame cannot be delivered.
    pub fn write_frame(&self, envelope: &Envelope) -> io::Result<()> {
        write_frame(&self.stream, envelope)
    }

    /// Closes both ends of the shared socket.
    pub fn shutdown(&self) {
        let stream = lock(&self.stream);
        let _ = stream.shutdown(Shutdown::Both);
    }
}
