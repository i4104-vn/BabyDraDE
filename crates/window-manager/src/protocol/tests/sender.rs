//! Compositor-side sender tests over an in-memory socket pair.

use std::os::unix::net::UnixStream;
use std::time::Duration;

use common::protocol::{Kind, Target};

use crate::protocol::{BusSender, Connection, ReadResult};

#[test]
fn sender_publishes_a_retained_event_with_ttl() {
    let (writer_end, reader_end) = UnixStream::pair().expect("socket pair");
    let sender = BusSender::new(writer_end, "wm".to_string());
    let id = sender
        .publish_retained(
            "state",
            &serde_json::json!({"ready": true}),
            Duration::from_secs(5),
        )
        .expect("publish");
    let mut connection = Connection::new(reader_end);
    let messages = match connection.read_available().expect("read") {
        ReadResult::Messages(messages) => messages,
        _ => panic!("expected a frame"),
    };
    let envelope = messages.first().expect("envelope");
    assert_eq!(envelope.id, id);
    assert_eq!(envelope.target, Target::Topic("state".to_string()));
    assert!(envelope.retained);
    assert_eq!(envelope.ttl_ms, Some(5_000));
}

#[test]
fn sender_writes_subscribe_control_and_peer_request_frames() {
    let (writer_end, reader_end) = UnixStream::pair().expect("socket pair");
    let sender = BusSender::new(writer_end, "wm".to_string());
    sender.subscribe("state").expect("subscribe");
    let id = sender
        .request("feature", &serde_json::json!("ping"))
        .expect("request");
    let mut connection = Connection::new(reader_end);
    let messages = match connection.read_available().expect("read") {
        ReadResult::Messages(messages) => messages,
        _ => panic!("expected frames"),
    };
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].target, Target::Router);
    assert_eq!(messages[0].kind, Kind::Request);
    assert_eq!(messages[1].id, id);
    assert_eq!(messages[1].target, Target::Peer("feature".to_string()));
    assert_eq!(messages[1].kind, Kind::Request);
}
