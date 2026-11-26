//! Abstract Linux Unix socket used by the in-memory bus.
//!
//! The address is derived from the effective uid, so every process of the
//! session finds the same listener without creating filesystem artifacts.

use std::io;
use std::os::fd::AsRawFd;
use std::os::linux::net::SocketAddrExt;
use std::os::unix::net::{SocketAddr, UnixStream};

/// Builds the uid-scoped abstract address of the bus.
///
/// # Returns
/// The abstract socket address for the current effective uid.
///
/// # Errors
/// Returns an error when the generated name cannot be represented as an
/// abstract address.
pub fn abstract_addr() -> io::Result<SocketAddr> {
    let uid = unsafe { libc::geteuid() };
    SocketAddr::from_abstract_name(format!("babydra-bus-{uid}").as_bytes())
}

/// Connects to the bus listener of the current uid.
///
/// # Errors
/// Returns the socket error when no compositor is listening.
pub fn connect() -> io::Result<UnixStream> {
    UnixStream::connect_addr(&abstract_addr()?)
}

/// Verifies that the other end of a connection runs as the current uid.
///
/// `UnixStream::peer_cred` is still unstable, so the Linux `SO_PEERCRED`
/// socket option is queried directly.
///
/// # Arguments
/// * `stream` - Accepted or connected stream to check.
///
/// # Returns
/// `Ok(())` when the peer runs as the current effective uid.
///
/// # Errors
/// Returns the `getsockopt` error or [`io::ErrorKind::PermissionDenied`] when
/// the peer uid differs from the effective uid.
pub fn verify_peer(stream: &UnixStream) -> io::Result<()> {
    let mut cred: libc::ucred = unsafe { std::mem::zeroed() };
    let mut len = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
    let result = unsafe {
        libc::getsockopt(
            stream.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            (&mut cred as *mut libc::ucred).cast::<libc::c_void>(),
            &mut len,
        )
    };
    if result != 0 {
        return Err(io::Error::last_os_error());
    }
    let uid = unsafe { libc::geteuid() };
    if cred.uid != uid {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "bus peer has a different uid",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abstract_address_is_scoped_to_the_current_uid() {
        let address = abstract_addr().expect("abstract address");
        let name = address.as_abstract_name().expect("abstract name");
        let uid = unsafe { libc::geteuid() };
        assert_eq!(name, format!("babydra-bus-{uid}").as_bytes());
    }

    #[test]
    fn accepts_a_peer_with_the_same_uid() {
        let (left, right) = UnixStream::pair().expect("socket pair");
        assert!(verify_peer(&left).is_ok());
        drop(right);
    }
}
