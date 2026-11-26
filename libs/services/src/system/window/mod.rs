//! Active window tracking and window actions for BabyDraDE features.
//!
//! Focused-window metadata is published as a retained `window.active` event on
//! the in-memory protocol bus. Window actions are requests to the compositor's
//! `wm` peer. Peers exchange framed JSON over a uid-scoped abstract Unix socket;
//! a router in the compositor owns delivery and retention. No files or queue
//! directories are involved.

use std::io;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use crate::protocol::{Bus, Event, Incoming, Request, WindowAction, WindowTarget};

pub use common::protocol::ActiveWindowInfo;

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

/// Formats a user-friendly application name from app id and title.
///
/// # Arguments
/// * `app_id` - Wayland app id, preferred when it carries a usable name.
/// * `title` - Window title, used when the app id is empty.
///
/// # Returns
/// A capitalised app name, or the title's trailing app part as a fallback.
pub fn get_app_display_name(app_id: &str, title: &str) -> String {
    let trimmed_id = app_id
        .trim()
        .strip_suffix(".desktop")
        .unwrap_or(app_id.trim());
    if !trimmed_id.is_empty() {
        let name = trimmed_id.split('.').next_back().unwrap_or(trimmed_id);
        let mut chars = name.chars();
        if let Some(first) = chars.next() {
            return format!("{}{}", first.to_uppercase(), chars.as_str());
        }
    }
    let trimmed_title = title.trim();
    if let Some((_, app_part)) = trimmed_title.rsplit_once(" — ") {
        return app_part.trim().to_string();
    }
    if let Some((_, app_part)) = trimmed_title.rsplit_once(" - ") {
        return app_part.trim().to_string();
    }
    trimmed_title.to_string()
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
fn send_request(request: &Request) {
    if let Ok(bus) = bus() {
        let _ = bus.request("wm", request);
    }
}

/// Sends a window action through the protocol bus.
///
/// # Arguments
/// * `action` - Legacy action name (`close`, `minimize`, `maximize`, `fullscreen`).
pub fn send_window_action(action: &str) {
    let Some(action) = WindowAction::parse(action) else {
        return;
    };
    send_request(&Request::WindowAction { action });
}

/// Closes the window identified by `app_id` or `title`, or the focused one.
///
/// # Arguments
/// * `app_id` - Wayland app id to match, preferred when non-empty.
/// * `title` - Window title to match when `app_id` is empty.
pub fn close_window_by_app(app_id: &str, title: &str) {
    let target = if !app_id.is_empty() {
        WindowTarget::AppId(app_id.to_string())
    } else if !title.is_empty() {
        WindowTarget::Title(title.to_string())
    } else {
        WindowTarget::Focused
    };
    send_request(&Request::CloseTargeted { target });
}

/// Minimizes the currently focused window.
pub fn minimize_active_window() {
    send_request(&Request::WindowAction {
        action: WindowAction::Minimize,
    });
}

/// Maximizes the currently focused window.
pub fn maximize_active_window() {
    send_request(&Request::WindowAction {
        action: WindowAction::Maximize,
    });
}

/// Toggles fullscreen on the currently focused window.
pub fn toggle_fullscreen_active_window() {
    send_request(&Request::WindowAction {
        action: WindowAction::Fullscreen,
    });
}
