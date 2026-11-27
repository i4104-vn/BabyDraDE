//! Retained `window.active` bus subscription and compositor requests.

use std::io;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use crate::protocol::{Bus, Event, Incoming, Request};

use super::state::{active_state, throttle_due, ActiveState};

/// Minimum delay between two attempts to start a bus watcher.
const CONNECT_INTERVAL: Duration = Duration::from_secs(2);

/// Connects (once) to the protocol bus as the services peer.
///
/// # Returns
/// The shared bus handle.
///
/// # Errors
/// Returns the socket error raised while connecting.
fn bus() -> io::Result<&'static Bus> {
    static BUS: OnceLock<Bus> = OnceLock::new();
    if BUS.get().is_none() {
        let _ = BUS.set(Bus::connect("org.babydrade.services")?);
    }
    BUS.get()
        .ok_or_else(|| io::Error::other("protocol bus unavailable"))
}

/// Starts at most one background watcher at a time.
///
/// Connection and subscription happen on the worker thread: polling widgets
/// must never block on the bus handshake. A failed attempt is retried, but no
/// more often than `CONNECT_INTERVAL`.
pub(super) fn ensure_active_watcher(state: &mut ActiveState, now: Instant) {
    if state.watcher_pending || !throttle_due(state.watcher_at, now, CONNECT_INTERVAL) {
        return;
    }
    state.watcher_at = Some(now);
    let spawned = std::thread::Builder::new()
        .name("babydra-active-watcher".into())
        .spawn(watcher_loop);
    match spawned {
        Ok(_) => state.watcher_pending = true,
        Err(_) => {
            state.watcher_pending = false;
            state.watcher_at = None;
        }
    }
}

/// Feeds the active-window cache from the retained bus topic.
///
/// A reconnect is left to the next throttled `ensure_active_watcher()` call;
/// until then the cached `wlrctl` snapshot keeps the UI stable.
fn watcher_loop() {
    let Ok(bus) = bus() else {
        watcher_stopped();
        return;
    };
    if bus.subscribe("window.active").is_err() {
        watcher_stopped();
        return;
    }
    loop {
        match bus.recv_timeout(Duration::from_secs(1)) {
            Some(Incoming::Event(message)) if message.topic.as_deref() == Some("window.active") => {
                if let Ok(Event::ActiveWindowChanged { info }) = message.decode::<Event>() {
                    let mut state = active_state().lock().unwrap_or_else(|err| {
                        tracing::warn!("active window state lock poisoned; recovering");
                        err.into_inner()
                    });
                    state.bus_info = Some(info);
                }
            }
            Some(Incoming::Disconnected) => {
                watcher_stopped();
                break;
            }
            _ => {}
        }
    }
}

/// Marks the watcher as gone so throttled fallback probing can resume.
fn watcher_stopped() {
    let mut state = active_state().lock().unwrap_or_else(|err| {
        tracing::warn!("active window state lock poisoned; recovering");
        err.into_inner()
    });
    state.watcher_pending = false;
    state.bus_info = None;
}

/// Sends one request to the compositor's `wm` peer, ignoring transport errors.
///
/// # Arguments
/// * `request` - The window-manager service request to deliver.
pub(crate) fn send_request(request: &Request) {
    if let Ok(bus) = bus() {
        let _ = bus.request("wm", request);
    }
}
