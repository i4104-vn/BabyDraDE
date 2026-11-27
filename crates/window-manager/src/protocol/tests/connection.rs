//! Connection framing tests over an in-memory socket pair.

use std::io::{ErrorKind, Write};
use std::os::unix::net::UnixStream;

use crate::protocol::{Connection, ReadResult};

use super::common::connection_frame;

#[test]
fn decodes_queued_frames_and_reports_would_block() {
    let (sender, receiver) = UnixStream::pair().expect("socket pair");
    let mut writer = Connection::new(sender);
    let mut reader = Connection::new(receiver);
    assert!(writer.queue(&connection_frame(1)).is_ok());
    assert!(writer.queue(&connection_frame(2)).is_ok());
    assert!(writer.flush().expect("flush"));
    assert!(!writer.has_outbox());
    let messages = match reader.read_available().expect("read") {
        ReadResult::Messages(messages) => messages,
        _ => panic!("expected messages"),
    };
    assert_eq!(messages.len(), 2);
    assert_eq!(
        reader.read_available().expect("read"),
        ReadResult::WouldBlock
    );
}

#[test]
fn reports_invalid_data_for_a_malformed_frame() {
    let (mut sender, receiver) = UnixStream::pair().expect("socket pair");
    sender.write_all(b"not json\n").expect("write");
    let mut reader = Connection::new(receiver);
    assert!(
        matches!(reader.read_available(), Err(error) if error.kind() == ErrorKind::InvalidData)
    );
}

#[test]
fn reports_a_closed_peer_after_buffered_frames() {
    let (sender, receiver) = UnixStream::pair().expect("socket pair");
    let mut writer = Connection::new(sender);
    let mut reader = Connection::new(receiver);
    writer.queue(&connection_frame(9)).expect("queue");
    assert!(writer.flush().expect("flush"));
    drop(writer);
    assert_eq!(
        reader.read_available().expect("read"),
        ReadResult::Messages(vec![connection_frame(9)])
    );
    assert_eq!(reader.read_available().expect("read"), ReadResult::Closed);
}
