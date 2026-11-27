//! Active-window tracking over the protocol bus, with a `wlrctl` fallback.

use std::io;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use crate::protocol::{Bus, Event, Incoming, Request};

use super::ActiveWindowInfo;

/// Queries the focused window, falling back to `wlrctl` when the bus has no
/// cached snapshot yet.
pub fn get_active_window() -> Option<ActiveWindowInfo> {
    ensure_active_watcher();
    active_cache()
        .lock()
        .ok()
        .and_then(|cache| cache.clone())
        .or_else(query_wlrctl_active_window)
}

/// Reads the focused window from `wlrctl` as a one-shot fallback.
///
/// # Returns
/// The focused window's app id and title, or `None` when `wlrctl` is absent or
/// reports no focused window.
fn query_wlrctl_active_window() -> Option<ActiveWindowInfo> {
    let output = Command::new("wlrctl")
        .args(["window", "list", "state:focused"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
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

/// Returns the in-memory cache of the last retained active-window event.
fn active_cache() -> &'static Mutex<Option<ActiveWindowInfo>> {
    static ACTIVE: OnceLock<Mutex<Option<ActiveWindowInfo>>> = OnceLock::new();
    ACTIVE.get_or_init(|| Mutex::new(None))
}

/// Starts the single background reader that feeds the active-window cache.
///
/// Each call retries while the compositor is unreachable; only the first
/// successful subscription spawns the reader, so one watcher feeds the cache
/// for the lifetime of the process.
fn ensure_active_watcher() {
    static SPAWNED: AtomicBool = AtomicBool::new(false);
    if SPAWNED.load(Ordering::Relaxed) {
        return;
    }

    // Retry until the compositor is reachable: a failed first attempt must not
    // permanently disable the cache.
    let Ok(bus) = bus() else {
        return;
    };
    if bus.subscribe("window.active").is_err() {
        return;
    }
    if SPAWNED.swap(true, Ordering::Relaxed) {
        return;
    }

    std::thread::spawn(move || loop {
        match bus.recv_timeout(Duration::from_secs(1)) {
            Some(Incoming::Event(message)) if message.topic.as_deref() == Some("window.active") => {
                if let Ok(Event::ActiveWindowChanged { info }) = message.decode::<Event>() {
                    if let Ok(mut cache) = active_cache().lock() {
                        *cache = Some(info);
                    }
                }
            }
            Some(Incoming::Disconnected) => break,
            _ => {}
        }
    });
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
