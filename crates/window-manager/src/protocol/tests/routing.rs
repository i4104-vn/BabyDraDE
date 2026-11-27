//! Router tests for request correlation, handshakes and control frames.

use common::protocol::{BusError, BusErrorCode, Control, Kind, Message, Target, PROTOCOL_VERSION};

use crate::protocol::Router;

use super::common::{connect, control_frame, errors, frame};

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
