//! Unit tests for the window-manager protocol module.

use std::io::{ErrorKind, Write};
use std::os::unix::net::UnixStream;
use std::time::{Duration, Instant};

use common::protocol::{
    BusError, BusErrorCode, Control, Envelope, Kind, Message, MessageId, PeerId, Target,
    PROTOCOL_VERSION,
};

use crate::protocol::{
    router::{OFFLINE_LIMIT, PEERS_TOPIC},
    BusSender, Connection, ReadResult, Router,
};

fn connect(router: &mut Router, name: &str) -> PeerId {
    let id = router.add_connection();
    router.hello(id, name.to_string()).expect("hello accepted");
    let _ = router.take_outbox(id);
    id
}

fn frame(id: MessageId, target: Target, kind: Kind, body: serde_json::Value) -> Envelope {
    Envelope::new(id, target, kind, body)
}

fn control_frame(id: MessageId, control: Control) -> Envelope {
    frame(
        id,
        Target::Router,
        Kind::Request,
        serde_json::to_value(control).expect("control payload"),
    )
}

fn errors(outbox: &[Envelope]) -> Vec<BusError> {
    outbox
        .iter()
        .filter(|envelope| envelope.kind == Kind::Error)
        .filter_map(|envelope| envelope.bus_error())
        .collect()
}

fn connection_frame(id: u64) -> Envelope {
    Envelope::new(id, Target::Broadcast, Kind::Event, serde_json::json!(id))
}

#[test]
fn routes_a_request_and_its_response_back_to_the_origin() {
    let mut router = Router::new();
    let a = connect(&mut router, "a");
    let b = connect(&mut router, "b");
    router.route(
        a,
        frame(
            11,
            Target::Peer("b".into()),
            Kind::Request,
            serde_json::json!(1),
        ),
    );
    let delivered = router.take_outbox(b);
    assert_eq!(
        Message::from_envelope(delivered[0].clone()).body,
        serde_json::json!(1)
    );
    assert_eq!(delivered[0].source, "a");
    assert_eq!(router.pending.len(), 1);
    router.route(
        b,
        frame(11, Target::Router, Kind::Response, serde_json::json!("ok")),
    );
    let response = router.take_outbox(a);
    assert_eq!(response.len(), 1);
    assert_eq!(response[0].source, "b");
    assert!(router.pending.is_empty());
}

#[test]
fn rejects_a_duplicate_peer_name() {
    let mut router = Router::new();
    connect(&mut router, "wm");
    let id = router.add_connection();
    let result = router.hello(id, "wm".to_string());
    assert_eq!(
        result,
        Err(BusError::new(
            BusErrorCode::NameTaken,
            "peer name 'wm' is already used by connection 1"
        ))
    );
    assert_eq!(
        errors(&router.take_outbox(id))[0].code,
        BusErrorCode::NameTaken
    );
}

#[test]
fn topic_events_reach_only_subscribers() {
    let mut router = Router::new();
    let publisher = connect(&mut router, "publisher");
    let listener = connect(&mut router, "listener");
    let outsider = connect(&mut router, "outsider");
    router
        .subscribe(listener, "state".into())
        .expect("subscribe");
    router.route(
        publisher,
        frame(
            1,
            Target::Topic("state".into()),
            Kind::Event,
            serde_json::json!(true),
        ),
    );
    assert_eq!(router.take_outbox(listener).len(), 1);
    assert!(router.take_outbox(outsider).is_empty());
    assert!(router.take_outbox(publisher).is_empty());
}

#[test]
fn replays_a_retained_event_to_a_late_subscriber_then_expires_it() {
    let mut router = Router::new();
    let publisher = connect(&mut router, "publisher");
    let mut envelope = frame(
        4,
        Target::Topic("state".into()),
        Kind::Event,
        serde_json::json!({"ready": true}),
    );
    envelope.retained = true;
    envelope.ttl_ms = Some(5_000);
    router.route(publisher, envelope);
    let late = connect(&mut router, "late");
    router.subscribe(late, "state".into()).expect("subscribe");
    let replayed = router.take_outbox(late);
    assert!(replayed[0].retained);
    router.tick(Instant::now() + Duration::from_secs(6));
    let newer = connect(&mut router, "newer");
    router.subscribe(newer, "state".into()).expect("subscribe");
    assert!(router.take_outbox(newer).is_empty());
}

#[test]
fn notifies_the_origin_when_the_target_disconnects() {
    let mut router = Router::new();
    let origin = connect(&mut router, "origin");
    let target = connect(&mut router, "target");
    router.route(
        origin,
        frame(
            3,
            Target::Peer("target".into()),
            Kind::Request,
            serde_json::json!({}),
        ),
    );
    let _ = router.take_outbox(target);
    router.remove(target);
    let outbox = router.take_outbox(origin);
    assert_eq!(errors(&outbox)[0].code, BusErrorCode::PeerUnavailable);
    assert!(router.pending.is_empty());
}

#[test]
fn times_out_an_unanswered_request_on_tick() {
    let mut router = Router::new();
    let origin = connect(&mut router, "origin");
    let target = connect(&mut router, "target");
    router.route(
        origin,
        frame(
            8,
            Target::Peer("target".into()),
            Kind::Request,
            serde_json::json!({}),
        ),
    );
    let _ = router.take_outbox(target);
    router.tick(Instant::now() + Duration::from_secs(11));
    let outbox = router.take_outbox(origin);
    assert_eq!(
        errors(&outbox),
        vec![BusError::new(
            BusErrorCode::Timeout,
            "request to 'target' timed out".to_string()
        )]
    );
}

#[test]
fn flushes_offline_events_on_hello() {
    let mut router = Router::new();
    let sender = connect(&mut router, "sender");
    let id = router.add_connection();
    router.route(
        sender,
        frame(
            2,
            Target::Peer("late".into()),
            Kind::Event,
            serde_json::json!(7),
        ),
    );
    router
        .hello(id, "late".to_string())
        .expect("hello accepted");
    let outbox = router.take_outbox(id);
    let event = outbox
        .iter()
        .find(|envelope| envelope.kind == Kind::Event)
        .expect("offline event");
    assert_eq!(
        Message::from_envelope(event.clone()).body,
        serde_json::json!(7)
    );
}

#[test]
fn announces_joins_and_leaves_on_the_peers_topic() {
    let mut router = Router::new();
    let watcher = connect(&mut router, "watcher");
    router
        .subscribe(watcher, PEERS_TOPIC.into())
        .expect("subscribe");
    let new_peer = connect(&mut router, "new_peer");
    router.remove(new_peer);
    let outbox = router.take_outbox(watcher);
    let controls: Vec<Control> = outbox
        .iter()
        .filter_map(|envelope| envelope.control())
        .collect();
    assert_eq!(
        controls,
        vec![
            Control::PeerJoined {
                name: "new_peer".into()
            },
            Control::PeerLeft {
                name: "new_peer".into()
            },
        ]
    );
}

#[test]
fn rejects_a_frame_with_an_unsupported_version() {
    let mut router = Router::new();
    let id = connect(&mut router, "a");
    let mut bad = frame(1, Target::Router, Kind::Request, serde_json::json!({}));
    bad.version = PROTOCOL_VERSION + 1;
    router.route(id, bad);
    let outbox = router.take_outbox(id);
    assert_eq!(errors(&outbox)[0].code, BusErrorCode::InvalidPayload);
}

#[test]
fn closes_a_connection_whose_first_frame_is_not_hello() {
    let mut router = Router::new();
    let id = router.add_connection();
    router.route(
        id,
        frame(1, Target::Router, Kind::Response, serde_json::json!({})),
    );
    let outbox = router.take_outbox(id);
    assert_eq!(errors(&outbox)[0].code, BusErrorCode::InvalidTarget);
}

#[test]
fn drops_frames_from_a_closing_connection() {
    let mut router = Router::new();
    let id = router.add_connection();
    router.route(
        id,
        frame(1, Target::Router, Kind::Event, serde_json::json!({})),
    );
    let _ = router.take_outbox(id);
    router.route(
        id,
        control_frame(
            2,
            Control::Hello {
                name: "late".into(),
            },
        ),
    );
    assert!(router.take_outbox(id).is_empty());
}

#[test]
fn rejects_a_request_addressed_to_a_topic() {
    let mut router = Router::new();
    let a = connect(&mut router, "a");
    router.route(
        a,
        frame(
            1,
            Target::Topic("t".into()),
            Kind::Request,
            serde_json::json!(1),
        ),
    );
    let outbox = router.take_outbox(a);
    assert_eq!(errors(&outbox)[0].code, BusErrorCode::InvalidTarget);
}

#[test]
fn rejects_an_event_addressed_to_the_router() {
    let mut router = Router::new();
    let a = connect(&mut router, "a");
    router.route(
        a,
        frame(2, Target::Router, Kind::Event, serde_json::json!(1)),
    );
    let outbox = router.take_outbox(a);
    assert_eq!(errors(&outbox)[0].code, BusErrorCode::InvalidPayload);
}

#[test]
fn subscribes_via_a_router_control_frame() {
    let mut router = Router::new();
    let publisher = connect(&mut router, "publisher");
    let listener = connect(&mut router, "listener");
    router.route(
        listener,
        control_frame(1, Control::Subscribe { topic: "t".into() }),
    );
    router.route(
        publisher,
        frame(
            2,
            Target::Topic("t".into()),
            Kind::Event,
            serde_json::json!(1),
        ),
    );
    assert_eq!(router.take_outbox(listener).len(), 1);
}

#[test]
fn unsubscribes_via_a_router_control_frame() {
    let mut router = Router::new();
    let publisher = connect(&mut router, "publisher");
    let listener = connect(&mut router, "listener");
    router.subscribe(listener, "t".into()).expect("subscribe");
    router.route(
        listener,
        control_frame(1, Control::Unsubscribe { topic: "t".into() }),
    );
    router.route(
        publisher,
        frame(
            2,
            Target::Topic("t".into()),
            Kind::Event,
            serde_json::json!(1),
        ),
    );
    assert!(router.take_outbox(listener).is_empty());
}

#[test]
fn rejects_an_unsupported_router_control_frame() {
    let mut router = Router::new();
    let a = connect(&mut router, "a");
    router.route(a, control_frame(1, Control::Hello { name: "a".into() }));
    let outbox = router.take_outbox(a);
    assert_eq!(errors(&outbox)[0].code, BusErrorCode::InvalidPayload);
}

#[test]
fn expires_queued_offline_events_on_tick() {
    let mut router = Router::new();
    let sender = connect(&mut router, "sender");
    let mut event = frame(
        1,
        Target::Peer("late".into()),
        Kind::Event,
        serde_json::json!(1),
    );
    event.ttl_ms = Some(100);
    router.route(sender, event);
    router.tick(Instant::now() + Duration::from_secs(1));
    let late = router.add_connection();
    router
        .hello(late, "late".to_string())
        .expect("hello accepted");
    assert!(router
        .take_outbox(late)
        .iter()
        .all(|envelope| envelope.kind != Kind::Event));
}

#[test]
fn evicts_the_oldest_offline_event_when_the_queue_is_full() {
    let mut router = Router::new();
    let sender = connect(&mut router, "sender");
    for id in 1..=OFFLINE_LIMIT + 1 {
        router.route(
            sender,
            frame(
                id as MessageId,
                Target::Peer("late".into()),
                Kind::Event,
                serde_json::json!(id),
            ),
        );
    }
    let late = router.add_connection();
    router
        .hello(late, "late".to_string())
        .expect("hello accepted");
    let events: Vec<Envelope> = router
        .take_outbox(late)
        .into_iter()
        .filter(|envelope| envelope.kind == Kind::Event)
        .collect();
    assert_eq!(
        Message::from_envelope(events[0].clone()).body,
        serde_json::json!(2)
    );
}

#[test]
fn drops_a_response_with_an_unknown_correlation_id() {
    let mut router = Router::new();
    let origin = connect(&mut router, "origin");
    let target = connect(&mut router, "target");
    router.route(
        origin,
        frame(
            7,
            Target::Peer("target".into()),
            Kind::Request,
            serde_json::json!({}),
        ),
    );
    let _ = router.take_outbox(target);
    router.route(
        target,
        frame(
            8,
            Target::Router,
            Kind::Response,
            serde_json::json!("wrong"),
        ),
    );
    assert!(router.take_outbox(origin).is_empty());
}

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
