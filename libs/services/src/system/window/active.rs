//! Active-window tracking over the protocol bus, with a `wlrctl` fallback.

use std::io;
use std::process::Command;
use std::sync::mpsc::{self, Receiver};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use crate::protocol::{Bus, Event, Incoming, Request};

use super::ActiveWindowInfo;

/// Longest time a `wlrctl` snapshot is reused before another probe is allowed.
const FALLBACK_INTERVAL: Duration = Duration::from_secs(1);
/// Minimum delay between two attempts to start a bus watcher.
const CONNECT_INTERVAL: Duration = Duration::from_secs(2);
/// Backoff when `wlrctl` could not be executed at all (for example not installed).
const UNAVAILABLE_BACKOFF: Duration = Duration::from_secs(30);
/// A probe older than this is treated as lost so a hung `wlrctl` cannot stall updates.
const PROBE_TIMEOUT: Duration = Duration::from_secs(5);

/// Pure timing rule shared by the fallback and watcher throttles.
///
/// `now` is injected so the rule can be tested without sleeping.
pub(crate) fn throttle_due(last: Option<Instant>, now: Instant, interval: Duration) -> bool {
    last.is_none_or(|started| now.saturating_duration_since(started) >= interval)
}

/// Pure decision on whether a fallback probe may start.
///
/// No process is spawned while a bus watcher can deliver a snapshot or while
/// another probe is in flight; otherwise the normal/unavailable backoff
/// applies. `now` is injected so the rule can be tested without sleeping.
pub(crate) fn fallback_probe_due(
    watcher_pending: bool,
    probe_pending: bool,
    unavailable: bool,
    last_probe: Option<Instant>,
    now: Instant,
) -> bool {
    !watcher_pending
        && !probe_pending
        && throttle_due(
            last_probe,
            now,
            if unavailable {
                UNAVAILABLE_BACKOFF
            } else {
                FALLBACK_INTERVAL
            },
        )
}

/// Parses the first `wlrctl window list` line into a focused-window snapshot.
pub(crate) fn parse_wlrctl_line(stdout: &str) -> Option<ActiveWindowInfo> {
    let line = stdout.lines().next()?;
    let (app_id, title) = line.split_once(':')?;
    let app_id = app_id.trim().to_string();
    let title = title.trim().to_string();
    if app_id.is_empty() && title.is_empty() {
        return None;
    }
    Some(ActiveWindowInfo {
        app_id,
        title,
        process_id: None,
    })
}

/// One-shot `wlrctl` probe. Errors mean the tool itself could not be executed.
fn probe_wlrctl() -> io::Result<Option<ActiveWindowInfo>> {
    let output = Command::new("wlrctl")
        .args(["window", "list", "state:focused"])
        .output()?;
    if !output.status.success() {
        return Ok(None);
    }
    Ok(parse_wlrctl_line(&String::from_utf8_lossy(&output.stdout)))
}

/// Outcome of a completed background fallback probe.
#[derive(Debug, Clone, PartialEq, Eq)]
enum ProbeResult {
    Snapshot(Option<ActiveWindowInfo>),
    Unavailable,
}

struct ActiveState {
    /// Snapshot delivered by the retained `window.active` bus event.
    bus_info: Option<ActiveWindowInfo>,
    /// A watcher worker is starting or still running for this process.
    watcher_pending: bool,
    /// Last snapshot supplied by the throttled `wlrctl` fallback.
    fallback: Option<ActiveWindowInfo>,
    /// When the last fallback probe was launched.
    probe_at: Option<Instant>,
    /// A fallback probe is currently running on a worker thread.
    probe_pending: bool,
    /// `wlrctl` could not be executed on the last probe.
    probe_unavailable: bool,
    /// Results from the most recent fallback probe.
    probe_results: Receiver<ProbeResult>,
    /// When the last watcher worker was launched.
    watcher_at: Option<Instant>,
}

impl ActiveState {
    fn new() -> Self {
        // A closed channel is harmless: results are only read when a probe is
        // actually in flight (see `collect_probe_result`).
        let (_tx, rx) = mpsc::channel();
        Self {
            bus_info: None,
            watcher_pending: false,
            fallback: None,
            probe_at: None,
            probe_pending: false,
            probe_unavailable: false,
            probe_results: rx,
            watcher_at: None,
        }
    }
}

/// Returns the shared active-window state.
fn active_state() -> &'static Mutex<ActiveState> {
    static ACTIVE: OnceLock<Mutex<ActiveState>> = OnceLock::new();
    ACTIVE.get_or_init(|| Mutex::new(ActiveState::new()))
}

/// Reads a finished probe result so the next poll can reuse it from cache.
fn collect_probe_result(state: &mut ActiveState, now: Instant) {
    if !state.probe_pending {
        return;
    }
    match state.probe_results.try_recv() {
        Ok(ProbeResult::Snapshot(info)) => {
            state.probe_pending = false;
            state.probe_unavailable = false;
            // A bus snapshot is authoritative; do not overwrite it with a
            // result that was already in flight when the event arrived.
            if state.bus_info.is_none() {
                state.fallback = info;
            }
        }
        Ok(ProbeResult::Unavailable) => {
            state.probe_pending = false;
            state.probe_unavailable = true;
        }
        Err(mpsc::TryRecvError::Disconnected) => state.probe_pending = false,
        Err(mpsc::TryRecvError::Empty) => {
            // Never wait on a probe: a stuck tool must not freeze the UI.
            if state
                .probe_at
                .is_some_and(|at| now.saturating_duration_since(at) >= PROBE_TIMEOUT)
            {
                state.probe_pending = false;
            }
        }
    }
}

/// Starts one `wlrctl` probe off the caller's thread.
///
/// The result is delivered through the state's channel and picked up by the
/// next `get_active_window()` call, so callers never spawn a process inline.
fn start_probe(state: &mut ActiveState, now: Instant) {
    let (tx, rx) = mpsc::channel();
    let spawned = std::thread::Builder::new()
        .name("babydra-wlrctl-active".into())
        .spawn(move || {
            let result = match probe_wlrctl() {
                Ok(info) => ProbeResult::Snapshot(info),
                Err(_) => ProbeResult::Unavailable,
            };
            let _ = tx.send(result);
        });
    if spawned.is_ok() {
        state.probe_results = rx;
        state.probe_pending = true;
        state.probe_at = Some(now);
    } else {
        // Retry on a later call rather than blocking the probe forever.
        state.probe_at = None;
    }
}

/// Queries the focused window from cached data, falling back to `wlrctl`.
///
/// Bus snapshots win as soon as they arrive. While a watcher is pending no
/// process is spawned at all; otherwise `wlrctl` runs at most once per
/// `FALLBACK_INTERVAL` (and much more rarely when the tool is unavailable).
/// Every caller sees the same cached snapshot between probes.
///
/// # Panics
/// Panics with `"active window state lock poisoned"` when another thread
/// panicked while holding the lock.
pub fn get_active_window() -> Option<ActiveWindowInfo> {
    let now = Instant::now();
    let mut state = active_state()
        .lock()
        .expect("active window state lock poisoned");
    collect_probe_result(&mut state, now);
    ensure_active_watcher(&mut state, now);

    if let Some(info) = state.bus_info.as_ref() {
        return Some(info.clone());
    }
    if fallback_probe_due(
        state.watcher_pending,
        state.probe_pending,
        state.probe_unavailable,
        state.probe_at,
        now,
    ) {
        start_probe(&mut state, now);
    }
    state.fallback.clone()
}

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
fn ensure_active_watcher(state: &mut ActiveState, now: Instant) {
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
                    let mut state = active_state()
                        .lock()
                        .expect("active window state lock poisoned");
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
    let mut state = active_state()
        .lock()
        .expect("active window state lock poisoned");
    state.watcher_pending = false;
    state.bus_info = None;
}

/// Sends one request to the compositor's `wm` peer, ignoring transport errors.
///
/// # Arguments
/// * `request` - The window-manager service request to deliver.
pub(super) fn send_request(request: &Request) {
    if let Ok(bus) = bus() {
        let _ = bus.request("wm", request);
    }
}
