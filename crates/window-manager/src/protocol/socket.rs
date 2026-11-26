//! Abstract nonblocking listener for the in-memory bus.
//!
//! The address derived by [`common::protocol::abstract_addr`] is scoped to the
//! effective uid, so the listener creates no filesystem artifacts.

use std::io;
use std::os::unix::net::UnixListener;

use common::protocol::abstract_addr;

/// Binds the bus listener in nonblocking mode.
///
/// # Returns
/// A listener ready to be polled by the compositor event loop.
///
/// # Errors
/// Returns the socket error when the address is invalid or already in use.
pub fn bind() -> io::Result<UnixListener> {
    let listener = UnixListener::bind_addr(&abstract_addr()?)?;
    listener.set_nonblocking(true)?;
    Ok(listener)
}
