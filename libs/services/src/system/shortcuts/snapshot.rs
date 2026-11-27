//! Shortcut table snapshots and manifest synthesis.

use std::collections::BTreeMap;
use std::collections::VecDeque;

use common::protocol::{Incoming, Request};
use common::ShortcutManifest;

use crate::protocol::Bus;

use super::error::ShortcutError;
use super::transport::{await_response, connect_bus, into_entries, io_error, WM_PEER};

/// Synthesized revision for bus snapshots: the compositor sends only a flat
/// binding list, so there is no real revision to carry over.
const SYNTHESIZED_REVISION: u64 = 1;

/// Reads all registered bindings and synthesizes one manifest per owner.
///
/// # Returns
///
/// Manifests ordered by owner; empty when the compositor cannot be reached.
/// Their `revision` is a synthesized constant because the bus carries only a
/// flat binding snapshot.
pub fn read_all() -> Vec<ShortcutManifest> {
    let bus = match connect_bus() {
        Ok(bus) => bus,
        Err(error) => {
            tracing::warn!("cannot connect to the shortcut bus: {error}");
            return Vec::new();
        }
    };
    let mut deferred = VecDeque::new();
    let manifests = list_all_entries(&bus, &mut deferred).map(group_manifests);
    bus.shutdown();
    match manifests {
        Ok(manifests) => manifests,
        Err(error) => {
            tracing::warn!("cannot read the shortcut list: {error}");
            Vec::new()
        }
    }
}

/// Requests the full binding table over one bus connection.
///
/// # Arguments
/// * `bus` - Connected feature bus.
/// * `deferred` - Queue for responses belonging to other requests.
///
/// # Returns
/// Every binding together with its owner id.
///
/// # Errors
/// Returns a bus, timeout or malformed-response failure.
fn list_all_entries(
    bus: &Bus,
    deferred: &mut VecDeque<Incoming>,
) -> Result<Vec<(String, common::ShortcutBinding)>, ShortcutError> {
    let id = bus
        .request(WM_PEER, &Request::ShortcutList)
        .map_err(io_error)?;
    into_entries(await_response(bus, deferred, id)?)
}

/// Keeps only the bindings belonging to `owner`.
///
/// # Arguments
/// * `entries` - Bindings paired with their owners.
/// * `owner` - Owner id to select.
///
/// # Returns
/// The selected bindings, in the compositor's order.
pub(crate) fn entries_for_owner(
    entries: Vec<(String, common::ShortcutBinding)>,
    owner: &str,
) -> Vec<common::ShortcutBinding> {
    entries
        .into_iter()
        .filter(|(entry_owner, _)| entry_owner == owner)
        .map(|(_, binding)| binding)
        .collect()
}

/// Groups a flat binding table into manifests, ordered by owner.
///
/// # Arguments
/// * `entries` - Bindings paired with their owners.
///
/// # Returns
/// One synthesized manifest per owner.
pub(crate) fn group_manifests(
    entries: Vec<(String, common::ShortcutBinding)>,
) -> Vec<ShortcutManifest> {
    let mut grouped: BTreeMap<String, Vec<common::ShortcutBinding>> = BTreeMap::new();
    for (owner, binding) in entries {
        grouped.entry(owner).or_default().push(binding);
    }
    grouped
        .into_iter()
        .map(|(owner, bindings)| ShortcutManifest {
            owner,
            revision: SYNTHESIZED_REVISION,
            bindings,
        })
        .collect()
}
