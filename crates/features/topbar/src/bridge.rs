//! Bridge from the topbar feature to the compositor.
//!
//! The GTK widget only calls the small helpers here. All protocol details
//! (queues, encoding, correlation ids) live in the `protocol` crate.

use std::cell::RefCell;

use protocol::{client::FeatureBridge, models::WindowAction};

/// Stable feature id used to namespace shortcut bindings and requests.
pub const OWNER: &str = "org.babydrade.topbar";

thread_local! {
    static BRIDGE: RefCell<Option<FeatureBridge>> = const { RefCell::new(None) };
}

/// Runs `f` with this process's bridge, creating it on first use.
fn with_bridge<R>(f: impl FnOnce(&FeatureBridge) -> R) -> R {
    BRIDGE.with(|slot| {
        let mut slot = slot.borrow_mut();
        if slot.is_none() {
            *slot = Some(FeatureBridge::new(OWNER));
        }
        let bridge = slot.as_ref().expect("bridge just initialized");
        f(bridge)
    })
}

/// Requests a window lifecycle action from the compositor.
pub fn window_action(action: WindowAction) {
    with_bridge(|bridge| {
        let _ = bridge.window_action(action);
    });
}

/// Returns compositor events received since the last call.
pub fn poll_events() -> Vec<protocol::models::Event> {
    with_bridge(|bridge| bridge.poll_events())
}
