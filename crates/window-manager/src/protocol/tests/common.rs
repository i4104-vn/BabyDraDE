//! Shared fixtures for the protocol tests.

use common::protocol::{BusError, Control, Envelope, Kind, MessageId, PeerId, Target};

use crate::protocol::Router;

pub(super) fn connect(router: &mut Router, name: &str) -> PeerId {
    let id = router.add_connection();
    router.hello(id, name.to_string()).expect("hello accepted");
    let _ = router.take_outbox(id);
    id
}

pub(super) fn frame(
    id: MessageId,
    target: Target,
    kind: Kind,
    body: serde_json::Value,
) -> Envelope {
    Envelope::new(id, target, kind, body)
}

pub(super) fn control_frame(id: MessageId, control: Control) -> Envelope {
    frame(
        id,
        Target::Router,
        Kind::Request,
        serde_json::to_value(control).expect("control payload"),
    )
}

pub(super) fn errors(outbox: &[Envelope]) -> Vec<BusError> {
    outbox
        .iter()
        .filter(|envelope| envelope.kind == Kind::Error)
        .filter_map(|envelope| envelope.bus_error())
        .collect()
}

pub(super) fn connection_frame(id: u64) -> Envelope {
    Envelope::new(id, Target::Broadcast, Kind::Event, serde_json::json!(id))
}
