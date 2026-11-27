//! Router tests for topic delivery, retention, offline queues and expiry.

use std::time::{Duration, Instant};

use common::protocol::{
    BusError, BusErrorCode, Control, Envelope, Kind, Message, MessageId, Target,
};

use crate::protocol::{
    router::{OFFLINE_LIMIT, PEERS_TOPIC},
    Router,
};

use super::common::{connect, errors, frame};

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
