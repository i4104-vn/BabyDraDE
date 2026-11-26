//! Bridge from the topbar feature to the compositor.
//!
//! The GTK widgets only call the small helpers here. All protocol details —
//! the abstract-socket connection, framing, correlation ids and retained
//! topics — live in `services::protocol`, so features never link the
//! compositor itself.

use std::io;
use std::sync::OnceLock;

use services::protocol::{Bus, Event, Incoming, Request, WindowAction};

/// Stable feature id used to namespace shortcut bindings and requests.
pub const OWNER: &str = "org.babydrade.topbar";

/// Connects (once) to the protocol bus as the topbar peer.
///
/// # Returns
/// The shared bus handle.
///
/// # Errors
/// Returns the socket error raised while connecting.
fn bus() -> io::Result<&'static Bus> {
    static BUS: OnceLock<Bus> = OnceLock::new();
    if BUS.get().is_none() {
        let _ = BUS.set(Bus::connect(OWNER)?);
    }
    BUS.get()
        .ok_or_else(|| io::Error::other("protocol bus unavailable"))
}

/// Requests a window lifecycle action from the compositor.
///
/// # Arguments
/// * `action` - Lifecycle action to apply to the focused window.
pub fn window_action(action: WindowAction) {
    if let Ok(bus) = bus() {
        let _ = bus.request("wm", &Request::WindowAction { action });
    }
}

/// Returns compositor events received since the last call.
///
/// # Returns
/// Decoded events; malformed frames and non-event traffic are discarded.
pub fn poll_events() -> Vec<Event> {
    let Ok(bus) = bus() else {
        return Vec::new();
    };
    bus.drain()
        .into_iter()
        .filter_map(|incoming| match incoming {
            Incoming::Event(message) => message.decode::<Event>().ok(),
            _ => None,
        })
        .collect()
}

/// Discards pending bus traffic so the reader thread cannot grow an unbounded
/// inbox when no widget polls events.
pub fn drain() {
    if let Ok(bus) = bus() {
        let _ = bus.drain();
    }
}
