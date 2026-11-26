//! Pure shortcut client rule tests.

use common::protocol::{ProtocolError, Response};
use common::{ShortcutBinding, ShortcutTarget};

use super::error::ShortcutError;
use super::snapshot::{entries_for_owner, group_manifests};
use super::transport::{into_ack, into_removal, valid_owner};

/// Builds a deterministic binding for filtering tests.
fn binding(id: &str) -> ShortcutBinding {
    ShortcutBinding {
        id: id.to_string(),
        combo: "W-S-t".into(),
        target: ShortcutTarget::Command("kitty".into()),
        enabled: true,
    }
}

#[test]
fn blank_owners_are_rejected_before_sending_a_request() {
    assert!(!valid_owner("  "));
    assert!(valid_owner("org.babydrade.topbar"));
}

#[test]
fn unknown_binding_is_not_an_error_when_unregistering() {
    let response = Response::Error {
        request_id: 7,
        error: ProtocolError::NotFound,
    };
    assert_eq!(into_removal(response), Ok(false));
}

#[test]
fn rejection_keeps_the_compositor_detail() {
    let response = Response::Error {
        request_id: 7,
        error: ProtocolError::Failed("bad combo".into()),
    };
    assert_eq!(
        into_ack(response),
        Err(ShortcutError::Failed("bad combo".into()))
    );
}

#[test]
fn bindings_from_other_owners_are_discarded() {
    let entries = vec![
        ("other".into(), binding("theirs")),
        ("mine".into(), binding("ours")),
    ];
    let selected = entries_for_owner(entries, "mine");
    assert_eq!(selected, vec![binding("ours")]);
}

#[test]
fn manifests_are_grouped_by_owner_in_stable_order() {
    let entries = vec![
        ("zeta".into(), binding("z")),
        ("alpha".into(), binding("a")),
        ("zeta".into(), binding("z2")),
    ];
    let owners: Vec<_> = group_manifests(entries)
        .into_iter()
        .map(|manifest| (manifest.owner, manifest.bindings.len()))
        .collect();
    assert_eq!(owners, vec![("alpha".into(), 1), ("zeta".into(), 2)]);
}
